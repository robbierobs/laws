//! S3 update handlers
//!
//! Handles S3-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::{App, Message, ServiceAction};
use crate::app::messages::S3Action;
use crate::event::{AwsEvent, Event};

impl App {
    pub(super) async fn handle_refresh_s3(
        &mut self,
        clients: &crate::aws::client::AwsClients,
        event_tx: crate::app::EventSender,
    ) {
        let client = clients.s3.clone();
        let tx = event_tx.clone();
        
        // Settings for rate limiting the detail loading
        let delay_ms = self.config.s3_detail_delay_ms; // e.g. 50ms
        
        // Spawn the main refresh task
        let handle = tokio::spawn(async move {
            let service = crate::aws::s3::S3Service::new(client.clone());
            match service.list_buckets().await {
                Ok(buckets) => {
                    // 1. Notify that buckets are loaded
                    tx.send(Event::Aws(AwsEvent::S3BucketsLoaded(buckets.clone())))
                        .await
                        .ok();

                    // 2. Trigger detail loading for top buckets (rate limited)
                    // We send messages back to the main loop to spawn tracked tasks
                    for bucket in buckets.iter().take(20) {
                        if delay_ms > 0 {
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        }
                        
                        tx.send(Event::Message(Message::Service(ServiceAction::S3(
                            S3Action::LoadBucketDetails(bucket.name.clone())
                        )))).await.ok();
                    }
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                        .await
                        .ok();
                }
            }
        });
        
        self.tasks.spawn(task_keys::S3_REFRESH, handle);

        // Also refresh objects if inside a bucket
        if let Some(bucket) = &self.services.s3.current_bucket {
            let bucket_name = bucket.clone();
            let client = clients.s3.clone();
            let tx = event_tx.clone();
            let handle = tokio::spawn(async move {
                let service = crate::aws::s3::S3Service::new(client);
                match service.list_objects(&bucket_name).await {
                    Ok(objects) => {
                        tx.send(Event::Aws(AwsEvent::S3ObjectsLoaded(objects)))
                            .await
                            .ok();
                    }
                    Err(e) => {
                        tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                            .await
                            .ok();
                    }
                }
            });
            self.tasks.spawn(task_keys::S3_OBJECTS, handle);
        }
    }

    pub(super) fn handle_load_s3_objects(
        &mut self,
        bucket: String,
        event_tx: crate::app::EventSender,
    ) {
        self.services.s3.current_bucket = Some(bucket.clone());

        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.s3.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            let service = crate::aws::s3::S3Service::new(client);
            match service.list_objects(&bucket).await {
                Ok(objects) => {
                    tx.send(Event::Aws(AwsEvent::S3ObjectsLoaded(objects)))
                        .await
                        .ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                        .await
                        .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::S3_OBJECTS, handle);
    }

    pub(super) fn handle_delete_s3_object(
        &mut self,
        bucket: String,
        key: String,
        event_tx: crate::app::EventSender,
    ) {
        self.spawn_aws_task(event_tx, task_keys::S3_ACTION, move |clients, tx| async move {
            let service = crate::aws::s3::S3Service::new(clients.s3.clone());
            match service.delete_object(&bucket, &key).await {
                Ok(_) => {
                    let msg = format!("Deleted object {}/{}", bucket, key);
                    tx.send(Event::Aws(AwsEvent::ActionCompleted(msg)))
                        .await
                        .ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                        .await
                        .ok();
                }
            }
        });
    }

    pub(super) fn handle_download_s3_object(
        &mut self,
        bucket: String,
        key: String,
        open_mode: bool,
        event_tx: crate::app::EventSender,
    ) {
        self.spawn_aws_task(event_tx, task_keys::S3_ACTION, move |clients, tx| async move {
            let service = crate::aws::s3::S3Service::new(clients.s3.clone());
            match service.get_object(&bucket, &key).await {
                Ok(bytes) => {
                    Self::write_s3_object(tx, key, bytes, open_mode).await;
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                        .await
                        .ok();
                }
            }
        });
    }

    /// Phase 1: Download S3 object for editing (async)
    /// This downloads the file and sends an event when ready for editing
    pub(super) fn handle_edit_s3_object(
        &mut self,
        bucket: String,
        key: String,
        event_tx: crate::app::EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.s3.clone();
        let tx = event_tx.clone();

        let handle = tokio::spawn(async move {
            let service = crate::aws::s3::S3Service::new(client);
            
            // Download the object
            let bytes = match service.get_object(&bucket, &key).await {
                Ok(b) => b,
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                        .await
                        .ok();
                    return;
                }
            };

            // Write to temp file
            let filename = key.split('/').next_back().unwrap_or(&key).to_string();
            let temp_dir = std::env::temp_dir().join("lazy_aws");
            if let Err(e) = std::fs::create_dir_all(&temp_dir) {
                tx.send(Event::Aws(AwsEvent::Error(format!(
                    "Failed to create temp directory: {}",
                    e
                ))))
                .await
                .ok();
                return;
            }

            let file_path = temp_dir.join(&filename);
            if let Err(e) = std::fs::write(&file_path, &bytes) {
                tx.send(Event::Aws(AwsEvent::Error(format!(
                    "Failed to write temp file: {}",
                    e
                ))))
                .await
                .ok();
                return;
            }

            // Signal that file is ready for editing
            tx.send(Event::Aws(AwsEvent::S3ObjectReadyForEdit {
                bucket,
                key,
                path: file_path.to_string_lossy().to_string(),
            }))
            .await
            .ok();
        });

        self.tasks.spawn(task_keys::S3_ACTION, handle);
    }

    /// Phase 2: Open editor synchronously and upload changes (runs on main thread)
    /// This is called after S3ObjectReadyForEdit event is received
    pub fn handle_edit_s3_object_sync(
        &mut self,
        bucket: String,
        key: String,
        path: String,
        event_tx: crate::app::EventSender,
    ) {
        use std::sync::atomic::Ordering;
        
        // Pause input handling before opening editor
        self.input_paused.store(true, Ordering::SeqCst);
        
        // Open in editor (this blocks until editor closes)
        let editor_result = crate::utils::editor::open_in_editor(&path);
        
        // Resume input handling after editor closes
        self.input_paused.store(false, Ordering::SeqCst);
        
        // Force terminal redraw after editor
        self.needs_redraw = true;
        
        match editor_result {
            Ok(_) => {
                // Read the edited content and upload
                match std::fs::read(&path) {
                    Ok(edited_bytes) => {
                        // Spawn async task for upload
                        let Some(clients) = &self.aws_clients else {
                            return;
                        };
                        
                        self.loading = true;
                        let client = clients.s3.clone();
                        let tx = event_tx;
                        let bucket_clone = bucket.clone();
                        let key_clone = key.clone();
                        let path_clone = path.clone();
                        
                        let handle = tokio::spawn(async move {
                            let service = crate::aws::s3::S3Service::new(client);
                            match service.put_object(&bucket_clone, &key_clone, edited_bytes).await {
                                Ok(_) => {
                                    tx.send(Event::Aws(AwsEvent::S3ObjectEdited {
                                        bucket: bucket_clone,
                                        key: key_clone,
                                    }))
                                    .await
                                    .ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(format!(
                                        "Failed to upload edited object: {}",
                                        e
                                    ))))
                                    .await
                                    .ok();
                                }
                            }
                            // Clean up temp file
                            std::fs::remove_file(&path_clone).ok();
                        });
                        
                        self.tasks.spawn(task_keys::S3_ACTION, handle);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to read edited file: {}", e));
                        self.action_log.push(format!("[ERROR] Failed to read edited file: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to open editor: {}", e));
                self.action_log.push(format!("[ERROR] Failed to open editor: {}", e));
            }
        }
    }

    async fn write_s3_object(
        tx: crate::app::EventSender,
        key: String,
        bytes: Vec<u8>,
        open_mode: bool,
    ) {
        let filename = key.split('/').next_back().unwrap_or(&key).to_string();

        let target_dir = if open_mode {
            std::env::temp_dir().join("lazy_aws")
        } else {
            dirs::download_dir().unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join("Downloads"))
                    .unwrap_or_else(std::env::temp_dir)
            })
        };

        if let Err(e) = std::fs::create_dir_all(&target_dir) {
            tx.send(Event::Aws(AwsEvent::Error(format!(
                "Failed to create directory: {}",
                e
            ))))
            .await
            .ok();
            return;
        }

        let file_path = target_dir.join(&filename);

        match std::fs::write(&file_path, &bytes) {
            Ok(_) => {
                let path_str = file_path.to_string_lossy().to_string();
                if open_mode {
                    let content = if bytes.len() < 1024 * 1024 {
                        String::from_utf8(bytes).ok()
                    } else {
                        None
                    };
                    tx.send(Event::Aws(AwsEvent::S3ObjectOpened {
                        key,
                        path: path_str,
                        content,
                    }))
                    .await
                    .ok();
                } else {
                    tx.send(Event::Aws(AwsEvent::S3ObjectDownloaded {
                        key,
                        path: path_str,
                    }))
                    .await
                    .ok();
                }
            }
            Err(e) => {
                tx.send(Event::Aws(AwsEvent::Error(format!(
                    "Failed to write file: {}",
                    e
                ))))
                .await
                .ok();
            }
        }
    }

    pub(super) fn handle_load_bucket_details(
        &mut self,
        bucket_name: String,
        event_tx: crate::app::EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        let mut loading_details = crate::models::s3::S3BucketDetails::default();
        loading_details.loading = true;
        self.services
            .s3
            .bucket_details
            .insert(bucket_name.clone(), loading_details);
        self.detail_loading = true;

        let client = clients.s3.clone();
        let tx = event_tx;
        let bucket = bucket_name.clone();

        let handle = tokio::spawn(async move {
            let service = crate::aws::s3::S3Service::new(client);
            let details = service.get_bucket_details(&bucket).await;
            tx.send(Event::Aws(AwsEvent::S3BucketDetailsLoaded {
                bucket_name: bucket,
                details,
            }))
            .await
            .ok();
        });

        // Use a unique key per bucket so they don't cancel each other
        let task_key = format!("{}_{}", task_keys::S3_DETAILS, bucket_name);
        self.tasks.spawn(task_key, handle);
    }
}
