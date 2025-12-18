use crate::app::task_manager::task_keys;
use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};
use std::sync::atomic::Ordering;

impl App {
    pub fn handle_ecs_deregister_task_definition(
        &mut self,
        task_definition_arn: String,
        event_tx: EventSender,
    ) {
        let arn = task_definition_arn.clone();
        let td_short = task_definition_arn
            .split('/')
            .next_back()
            .unwrap_or(&task_definition_arn)
            .to_string();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_ACTION,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                match ecs_client.deregister_task_definition(&arn).await {
                    Ok(()) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(format!(
                            "Deregistered task definition {}",
                            td_short
                        )))))
                        .await
                        .ok();
                    }
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                            "Failed to deregister task definition {}: {}",
                            td_short, e
                        )))))
                        .await
                        .ok();
                    }
                }
            }
        );
    }

    /// Phase 1: Download task definition for editing (async)
    /// This downloads and serializes the task definition, then sends an event when ready for editing
    pub fn handle_ecs_edit_task_definition(
        &mut self,
        task_definition_arn: String,
        event_tx: EventSender,
    ) {
        let arn = task_definition_arn.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_ACTION,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());

                // Get the task definition
                let task_def = match ecs_client.describe_task_definition(&arn).await {
                    Ok(td) => td,
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                            "Failed to fetch task definition: {}",
                            e
                        )))))
                        .await
                        .ok();
                        return;
                    }
                };

                // Serialize to JSON for editing
                let json = match serde_json::to_string_pretty(&task_def) {
                    Ok(j) => j,
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                            "Failed to serialize task definition: {}",
                            e
                        )))))
                        .await
                        .ok();
                        return;
                    }
                };

                // Write to temp file
                let temp_dir = std::env::temp_dir().join("lazy_aws");
                if let Err(e) = std::fs::create_dir_all(&temp_dir) {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                        "Failed to create temp directory: {}",
                        e
                    )))))
                    .await
                    .ok();
                    return;
                }

                let filename = format!("{}_taskdef.json", task_def.family);
                let file_path = temp_dir.join(&filename);
                if let Err(e) = std::fs::write(&file_path, &json) {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                        "Failed to write temp file: {}",
                        e
                    )))))
                    .await
                    .ok();
                    return;
                }

                // Signal that file is ready for editing
                tx.send(Event::Aws(Box::new(AwsEvent::EcsTaskDefinitionReadyForEdit {
                    family: task_def.family.clone(),
                    path: file_path.to_string_lossy().to_string(),
                })))
                .await
                .ok();
            }
        );
    }

    /// Phase 2: Open editor synchronously and register changes (runs on main thread)
    /// This is called after EcsTaskDefinitionReadyForEdit event is received
    pub fn handle_edit_task_definition_sync(
        &mut self,
        family: String,
        path: String,
        event_tx: EventSender,
    ) {
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
                // Read the edited content and register
                match std::fs::read_to_string(&path) {
                    Ok(edited_json) => {
                        let family_clone = family.clone();
                        let path_clone = path.clone();

                        // Use helper to spawn registration task
                        self.spawn_aws_task(
                            event_tx,
                            task_keys::ECS_ACTION,
                            move |clients, tx| async move {
                                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                                match ecs_client.register_task_definition(&edited_json).await {
                                    Ok(new_arn) => {
                                        tx.send(Event::Aws(Box::new(AwsEvent::EcsTaskDefinitionEdited {
                                            family: family_clone,
                                            new_arn,
                                        })))
                                        .await
                                        .ok();
                                    }
                                    Err(e) => {
                                        tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                                            "Failed to register edited task definition: {}",
                                            e
                                        )))))
                                        .await
                                        .ok();
                                    }
                                }
                                // Clean up temp file
                                std::fs::remove_file(&path_clone).ok();
                            }
                        );
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

    pub fn handle_ecs_list_task_definitions(&mut self, family: String, event_tx: EventSender) {
        let fam = family.clone();
        
        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_REFRESH,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                match ecs_client.list_task_definitions(Some(&fam)).await {
                    Ok(task_defs) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::EcsTaskDefinitionsListed(task_defs))))
                            .await
                            .ok();
                    }
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                            "Failed to list task definitions: {}",
                            e
                        )))))
                        .await
                        .ok();
                    }
                }
            }
        );
    }

    pub fn handle_ecs_update_service_task_definition(
        &mut self,
        cluster_arn: String,
        service_name: String,
        task_definition_arn: String,
        event_tx: EventSender,
    ) {
        let cluster = cluster_arn.clone();
        let service = service_name.clone();
        let td = task_definition_arn.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_ACTION,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                
                // Update the service to use the new task definition
                // Accessing underlying client for specific call format
                // Note: ecs_client.client must be accessible
                match ecs_client.client.update_service()
                    .cluster(&cluster)
                    .service(&service)
                    .task_definition(&td)
                    .send()
                    .await
                {
                    Ok(_) => {
                        let short_td = td.split('/').next_back().unwrap_or(&td);
                        tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(format!(
                            "Updated {} to use task definition {}",
                            service,
                            short_td
                        )))))
                        .await
                        .ok();

                        // Refresh services list
                        match ecs_client.list_services(&cluster).await {
                            Ok(services) => {
                                tx.send(Event::Aws(Box::new(AwsEvent::EcsServicesLoaded(services))))
                                    .await
                                    .ok();
                            }
                            Err(e) => {
                                tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                                    .await
                                    .ok();
                            }
                        }
                    }
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                            "Failed to update service: {}",
                            e
                        )))))
                        .await
                        .ok();
                    }
                }
            }
        );
    }

    pub fn handle_load_task_definitions_for_selector(
        &mut self,
        family: String,
        event_tx: EventSender,
    ) {
        let fam = family.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_REFRESH,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                match ecs_client.list_task_definitions_with_details(Some(&fam), Some(15)).await {
                    Ok(task_defs) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::EcsTaskDefinitionsForSelectorLoaded(task_defs))))
                            .await
                            .ok();
                    }
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                            "Failed to load task definitions: {}",
                            e
                        )))))
                        .await
                        .ok();
                    }
                }
            }
        );
    }
}
