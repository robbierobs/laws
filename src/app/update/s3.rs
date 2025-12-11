//! S3 update handlers
//!
//! Handles S3-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use crate::event::{AwsEvent, Event};

impl App {
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
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.s3.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            let service = crate::aws::s3::S3Service::new(client);
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

        self.tasks.spawn(task_keys::S3_ACTION, handle);
    }

    pub(super) fn handle_download_s3_object(
        &mut self,
        bucket: String,
        key: String,
        open_mode: bool,
        event_tx: crate::app::EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.s3.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            let service = crate::aws::s3::S3Service::new(client);
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

        self.tasks.spawn(task_keys::S3_ACTION, handle);
    }

    async fn write_s3_object(
        tx: crate::app::EventSender,
        key: String,
        bytes: Vec<u8>,
        open_mode: bool,
    ) {
        let filename = key.split('/').last().unwrap_or(&key).to_string();

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

        self.tasks.spawn(task_keys::S3_DETAILS, handle);
    }
}
