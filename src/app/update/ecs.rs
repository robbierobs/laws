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
}
