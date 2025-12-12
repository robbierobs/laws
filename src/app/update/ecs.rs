//! ECS update handlers
//!
//! Handles ECS-specific state mutations and async operations.

use crate::app::messages::{EcsAction, EcsViewMode};
use crate::app::task_manager::task_keys;
use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};

impl App {
    pub async fn handle_ecs_action(&mut self, action: EcsAction, event_tx: EventSender) {
        match action {
            // ================================================================
            // Navigation Actions
            // ================================================================
            EcsAction::ViewServices(cluster_arn) => {
                self.handle_ecs_view_services(cluster_arn, event_tx).await;
            }
            EcsAction::ViewTasks(service_arn) => {
                self.handle_ecs_view_tasks(service_arn, event_tx).await;
            }
            EcsAction::ViewTaskDefinition(task_definition_arn) => {
                self.handle_ecs_view_task_definition(task_definition_arn, event_tx)
                    .await;
            }
            EcsAction::BackToClusters => {
                self.handle_ecs_back_to_clusters();
            }
            EcsAction::BackToServices => {
                self.handle_ecs_back_to_services();
            }
            EcsAction::BackToTasks => {
                self.handle_ecs_back_to_tasks();
            }

            // ================================================================
            // Service Actions
            // ================================================================
            EcsAction::UpdateDesiredCount {
                cluster_arn,
                service_name,
                desired_count,
            } => {
                self.handle_ecs_update_desired_count(
                    cluster_arn,
                    service_name,
                    desired_count,
                    event_tx,
                )
                .await;
            }
            EcsAction::ForceNewDeployment {
                cluster_arn,
                service_name,
            } => {
                self.handle_ecs_force_new_deployment(cluster_arn, service_name, event_tx)
                    .await;
            }

            // ================================================================
            // Task Actions
            // ================================================================
            EcsAction::StopTask {
                cluster_arn,
                task_arn,
            } => {
                self.handle_ecs_stop_task(cluster_arn, task_arn, event_tx)
                    .await;
            }

            // ================================================================
            // Task Definition Actions
            // ================================================================
            EcsAction::DeregisterTaskDefinition(task_definition_arn) => {
                self.handle_ecs_deregister_task_definition(task_definition_arn, event_tx)
                    .await;
            }
            EcsAction::EditTaskDefinition(task_definition_arn) => {
                self.handle_ecs_edit_task_definition(task_definition_arn, event_tx)
                    .await;
            }
            EcsAction::ListTaskDefinitions(family) => {
                self.handle_ecs_list_task_definitions(family, event_tx)
                    .await;
            }
            EcsAction::UpdateServiceTaskDefinition {
                cluster_arn,
                service_name,
                task_definition_arn,
            } => {
                self.handle_ecs_update_service_task_definition(
                    cluster_arn,
                    service_name,
                    task_definition_arn,
                    event_tx,
                )
                .await;
            }
            EcsAction::UpdateService {
                cluster_arn,
                service_name,
                task_definition,
                cpu,
                memory,
                force_new_deployment,
            } => {
                self.handle_ecs_update_service(
                    cluster_arn,
                    service_name,
                    task_definition,
                    cpu,
                    memory,
                    force_new_deployment,
                    event_tx,
                )
                .await;
            }
            EcsAction::LoadTaskDefinitionsForSelector(family) => {
                self.handle_load_task_definitions_for_selector(family, event_tx).await;
            }
        }
    }

    // ========================================================================
    // Navigation Handlers
    // ========================================================================

    async fn handle_ecs_view_services(&mut self, cluster_arn: String, event_tx: EventSender) {
        self.services.ecs.selected_cluster_arn = Some(cluster_arn.clone());
        self.services.ecs.view_mode = EcsViewMode::Services;
        self.services.ecs.services.clear();
        self.services.ecs.list_state.select(None);
        self.loading = true;

        let Some(clients) = &self.aws_clients else {
            return;
        };

        let client = clients.ecs.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client);
            match ecs_client.list_services(&cluster_arn).await {
                Ok(services) => {
                    tx.send(Event::Aws(AwsEvent::EcsServicesLoaded(services)))
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

        self.tasks.spawn(task_keys::ECS_SERVICES, handle);
    }

    async fn handle_ecs_view_tasks(&mut self, service_arn: String, event_tx: EventSender) {
        // Extract service name from ARN for display
        let service_name = service_arn
            .split('/')
            .last()
            .unwrap_or(&service_arn)
            .to_string();

        self.services.ecs.selected_service_arn = Some(service_arn);
        self.services.ecs.selected_service_name = Some(service_name.clone());
        self.services.ecs.view_mode = EcsViewMode::Tasks;
        self.services.ecs.tasks.clear();
        self.services.ecs.list_state.select(None);
        self.loading = true;

        let Some(clients) = &self.aws_clients else {
            return;
        };

        let Some(cluster_arn) = self.services.ecs.selected_cluster_arn.clone() else {
            return;
        };

        let client = clients.ecs.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client);
            match ecs_client
                .list_tasks(&cluster_arn, Some(&service_name))
                .await
            {
                Ok(tasks) => {
                    tx.send(Event::Aws(AwsEvent::EcsTasksLoaded(tasks)))
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

        self.tasks.spawn(task_keys::ECS_TASKS, handle);
    }

    async fn handle_ecs_view_task_definition(
        &mut self,
        task_definition_arn: String,
        event_tx: EventSender,
    ) {
        self.services.ecs.view_mode = EcsViewMode::TaskDefinition;
        self.services.ecs.current_task_definition = None;
        self.services.ecs.detail_scroll_offset = 0;
        self.loading = true;

        let Some(clients) = &self.aws_clients else {
            return;
        };

        let client = clients.ecs.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client);
            match ecs_client
                .describe_task_definition(&task_definition_arn)
                .await
            {
                Ok(td) => {
                    tx.send(Event::Aws(AwsEvent::EcsTaskDefinitionLoaded(td)))
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

        self.tasks.spawn(task_keys::ECS_TASK_DEFINITION, handle);
    }

    fn handle_ecs_back_to_clusters(&mut self) {
        self.services.ecs.view_mode = EcsViewMode::Clusters;
        self.services.ecs.selected_cluster_arn = None;
        self.services.ecs.clear_services();
        self.services.ecs.clear_tasks();

        // Restore selection on clusters list
        if !self.services.ecs.clusters.is_empty()
            && self.services.ecs.list_state.selected().is_none()
        {
            self.services.ecs.list_state.select(Some(0));
        }
    }

    fn handle_ecs_back_to_services(&mut self) {
        self.services.ecs.view_mode = EcsViewMode::Services;
        self.services.ecs.clear_tasks();

        // Restore selection on services list
        if !self.services.ecs.services.is_empty()
            && self.services.ecs.list_state.selected().is_none()
        {
            self.services.ecs.list_state.select(Some(0));
        }
    }

    fn handle_ecs_back_to_tasks(&mut self) {
        self.services.ecs.view_mode = EcsViewMode::Tasks;
        self.services.ecs.current_task_definition = None;

        // Restore selection on tasks list
        if !self.services.ecs.tasks.is_empty() && self.services.ecs.list_state.selected().is_none()
        {
            self.services.ecs.list_state.select(Some(0));
        }
    }

    // ========================================================================
    // Service Action Handlers
    // ========================================================================

    async fn handle_ecs_update_desired_count(
        &mut self,
        cluster_arn: String,
        service_name: String,
        desired_count: i32,
        event_tx: EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.ecs.clone();
        let tx = event_tx.clone();
        let cluster_arn_clone = cluster_arn.clone();
        let service_name_clone = service_name.clone();

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client.clone());
            match ecs_client
                .update_service_desired_count(&cluster_arn, &service_name, desired_count)
                .await
            {
                Ok(()) => {
                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!(
                        "Updated {} desired count to {}",
                        service_name, desired_count
                    ))))
                    .await
                    .ok();

                    // Refresh services list
                    match ecs_client.list_services(&cluster_arn_clone).await {
                        Ok(services) => {
                            tx.send(Event::Aws(AwsEvent::EcsServicesLoaded(services)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(format!(
                        "Failed to update {}: {}",
                        service_name_clone, e
                    ))))
                    .await
                    .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::ECS_ACTION, handle);
    }

    async fn handle_ecs_force_new_deployment(
        &mut self,
        cluster_arn: String,
        service_name: String,
        event_tx: EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.ecs.clone();
        let tx = event_tx.clone();
        let cluster_arn_clone = cluster_arn.clone();
        let service_name_clone = service_name.clone();

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client.clone());
            match ecs_client
                .force_new_deployment(&cluster_arn, &service_name)
                .await
            {
                Ok(()) => {
                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!(
                        "Forced new deployment for {}",
                        service_name
                    ))))
                    .await
                    .ok();

                    // Refresh services list
                    match ecs_client.list_services(&cluster_arn_clone).await {
                        Ok(services) => {
                            tx.send(Event::Aws(AwsEvent::EcsServicesLoaded(services)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(format!(
                        "Failed to force deployment for {}: {}",
                        service_name_clone, e
                    ))))
                    .await
                    .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::ECS_ACTION, handle);
    }

    /// Handle updating an ECS service with optional task definition, CPU, and memory changes
    async fn handle_ecs_update_service(
        &mut self,
        cluster_arn: String,
        service_name: String,
        task_definition: Option<String>,
        cpu: Option<String>,
        memory: Option<String>,
        force_new_deployment: bool,
        event_tx: EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.ecs.clone();
        let tx = event_tx.clone();
        let cluster_arn_clone = cluster_arn.clone();
        let service_name_clone = service_name.clone();

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client.clone());
            
            // Build description for action completion message
            let mut changes = Vec::new();
            if task_definition.is_some() {
                changes.push("task definition".to_string());
            }
            if cpu.is_some() {
                changes.push(format!("CPU to {}", cpu.as_ref().unwrap()));
            }
            if memory.is_some() {
                changes.push(format!("memory to {}", memory.as_ref().unwrap()));
            }
            
            match ecs_client
                .update_service(
                    &cluster_arn,
                    &service_name,
                    task_definition.as_deref(),
                    cpu.as_deref(),
                    memory.as_deref(),
                    force_new_deployment,
                )
                .await
            {
                Ok(new_task_def_arn) => {
                    let short_arn = new_task_def_arn.split('/').last().unwrap_or(&new_task_def_arn);
                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!(
                        "Updated {} - {} (using {})",
                        service_name,
                        changes.join(", "),
                        short_arn
                    ))))
                    .await
                    .ok();

                    // Refresh services list
                    match ecs_client.list_services(&cluster_arn_clone).await {
                        Ok(services) => {
                            tx.send(Event::Aws(AwsEvent::EcsServicesLoaded(services)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(format!(
                        "Failed to update {}: {}",
                        service_name_clone, e
                    ))))
                    .await
                    .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::ECS_ACTION, handle);
    }

    // ========================================================================
    // Task Action Handlers
    // ========================================================================

    async fn handle_ecs_stop_task(
        &mut self,
        cluster_arn: String,
        task_arn: String,
        event_tx: EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.ecs.clone();
        let tx = event_tx.clone();
        let cluster_arn_clone = cluster_arn.clone();
        let service_name = self.services.ecs.selected_service_name.clone();
        let task_id = task_arn.split('/').last().unwrap_or(&task_arn).to_string();

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client.clone());
            match ecs_client
                .stop_task(&cluster_arn, &task_arn, "Stopped by LazyAWS user")
                .await
            {
                Ok(()) => {
                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!(
                        "Stopped task {}",
                        task_id
                    ))))
                    .await
                    .ok();

                    // Refresh tasks list
                    match ecs_client
                        .list_tasks(&cluster_arn_clone, service_name.as_deref())
                        .await
                    {
                        Ok(tasks) => {
                            tx.send(Event::Aws(AwsEvent::EcsTasksLoaded(tasks)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(format!(
                        "Failed to stop task {}: {}",
                        task_id, e
                    ))))
                    .await
                    .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::ECS_ACTION, handle);
    }

    // ========================================================================
    // Task Definition Action Handlers
    // ========================================================================

    async fn handle_ecs_deregister_task_definition(
        &mut self,
        task_definition_arn: String,
        event_tx: EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.ecs.clone();
        let tx = event_tx;
        let td_short = task_definition_arn
            .split('/')
            .last()
            .unwrap_or(&task_definition_arn)
            .to_string();

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client);
            match ecs_client
                .deregister_task_definition(&task_definition_arn)
                .await
            {
                Ok(()) => {
                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!(
                        "Deregistered task definition {}",
                        td_short
                    ))))
                    .await
                    .ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(format!(
                        "Failed to deregister task definition {}: {}",
                        td_short, e
                    ))))
                    .await
                    .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::ECS_ACTION, handle);
    }

    // ========================================================================
    // Edit Task Definition Handler
    // ========================================================================

    /// Phase 1: Download task definition for editing (async)
    /// This downloads and serializes the task definition, then sends an event when ready for editing
    async fn handle_ecs_edit_task_definition(
        &mut self,
        task_definition_arn: String,
        event_tx: EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.ecs.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client);

            // Get the task definition
            let task_def = match ecs_client
                .describe_task_definition(&task_definition_arn)
                .await
            {
                Ok(td) => td,
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(format!(
                        "Failed to fetch task definition: {}",
                        e
                    ))))
                    .await
                    .ok();
                    return;
                }
            };

            // Serialize to JSON for editing
            let json = match serde_json::to_string_pretty(&task_def) {
                Ok(j) => j,
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(format!(
                        "Failed to serialize task definition: {}",
                        e
                    ))))
                    .await
                    .ok();
                    return;
                }
            };

            // Write to temp file
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

            let filename = format!("{}_taskdef.json", task_def.family);
            let file_path = temp_dir.join(&filename);
            if let Err(e) = std::fs::write(&file_path, &json) {
                tx.send(Event::Aws(AwsEvent::Error(format!(
                    "Failed to write temp file: {}",
                    e
                ))))
                .await
                .ok();
                return;
            }

            // Signal that file is ready for editing
            tx.send(Event::Aws(AwsEvent::EcsTaskDefinitionReadyForEdit {
                family: task_def.family.clone(),
                path: file_path.to_string_lossy().to_string(),
            }))
            .await
            .ok();
        });

        self.tasks.spawn(task_keys::ECS_ACTION, handle);
    }

    /// Phase 2: Open editor synchronously and register changes (runs on main thread)
    /// This is called after EcsTaskDefinitionReadyForEdit event is received
    pub fn handle_edit_task_definition_sync(
        &mut self,
        family: String,
        path: String,
        event_tx: EventSender,
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
                // Read the edited content and register
                match std::fs::read_to_string(&path) {
                    Ok(edited_json) => {
                        // Spawn async task for registration
                        let Some(clients) = &self.aws_clients else {
                            return;
                        };
                        
                        self.loading = true;
                        let client = clients.ecs.clone();
                        let tx = event_tx;
                        let family_clone = family.clone();
                        let path_clone = path.clone();
                        
                        let handle = tokio::spawn(async move {
                            let ecs_client = crate::aws::ecs::EcsClient::new(client);
                            match ecs_client.register_task_definition(&edited_json).await {
                                Ok(new_arn) => {
                                    tx.send(Event::Aws(AwsEvent::EcsTaskDefinitionEdited {
                                        family: family_clone,
                                        new_arn,
                                    }))
                                    .await
                                    .ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(format!(
                                        "Failed to register edited task definition: {}",
                                        e
                                    ))))
                                    .await
                                    .ok();
                                }
                            }
                            // Clean up temp file
                            std::fs::remove_file(&path_clone).ok();
                        });
                        
                        self.tasks.spawn(task_keys::ECS_ACTION, handle);
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

    // ========================================================================
    // List Task Definitions Handler
    // ========================================================================

    async fn handle_ecs_list_task_definitions(&mut self, family: String, event_tx: EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.ecs.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client);
            match ecs_client.list_task_definitions(Some(&family)).await {
                Ok(task_defs) => {
                    tx.send(Event::Aws(AwsEvent::EcsTaskDefinitionsListed(task_defs)))
                        .await
                        .ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(format!(
                        "Failed to list task definitions: {}",
                        e
                    ))))
                    .await
                    .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::ECS_REFRESH, handle);
    }

    // ========================================================================
    // Update Service Task Definition Handler
    // ========================================================================

    async fn handle_ecs_update_service_task_definition(
        &mut self,
        cluster_arn: String,
        service_name: String,
        task_definition_arn: String,
        event_tx: EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.ecs.clone();
        let tx = event_tx.clone();
        let cluster_arn_clone = cluster_arn.clone();

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client.clone());
            
            // Update the service to use the new task definition
            match ecs_client.client.update_service()
                .cluster(&cluster_arn)
                .service(&service_name)
                .task_definition(&task_definition_arn)
                .send()
                .await
            {
                Ok(_) => {
                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!(
                        "Updated {} to use task definition {}",
                        service_name,
                        task_definition_arn.split('/').last().unwrap_or(&task_definition_arn)
                    ))))
                    .await
                    .ok();

                    // Refresh services list
                    match ecs_client.list_services(&cluster_arn_clone).await {
                        Ok(services) => {
                            tx.send(Event::Aws(AwsEvent::EcsServicesLoaded(services)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(format!(
                        "Failed to update service: {}",
                        e
                    ))))
                    .await
                    .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::ECS_ACTION, handle);
    }

    // ========================================================================
    // Load Task Definitions for Selector Handler
    // ========================================================================

    async fn handle_load_task_definitions_for_selector(
        &mut self,
        family: String,
        event_tx: EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.ecs.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            let ecs_client = crate::aws::ecs::EcsClient::new(client);
            match ecs_client.list_task_definitions_with_details(Some(&family), Some(15)).await {
                Ok(task_defs) => {
                    tx.send(Event::Aws(AwsEvent::EcsTaskDefinitionsForSelectorLoaded(task_defs)))
                        .await
                        .ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(format!(
                        "Failed to load task definitions: {}",
                        e
                    ))))
                    .await
                    .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::ECS_REFRESH, handle);
    }
}
