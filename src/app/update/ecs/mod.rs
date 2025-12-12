pub mod definitions;
pub mod navigation;
pub mod services;
pub mod tasks;

use crate::app::messages::EcsAction;
use crate::app::{App, EventSender};

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
}
