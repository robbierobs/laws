use crate::app::messages::EcsViewMode;
use crate::app::task_manager::task_keys;
use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};

impl App {
    pub fn handle_ecs_view_services(&mut self, cluster_arn: String, event_tx: EventSender) {
        self.services.ecs.selected_cluster_arn = Some(cluster_arn.clone());
        self.services.ecs.view_mode = EcsViewMode::Services;
        self.services.ecs.services.clear();
        self.services.ecs.list_state.select(None);
        
        let arn = cluster_arn.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_SERVICES,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                match ecs_client.list_services(&arn).await {
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
        );
    }

    pub fn handle_ecs_view_tasks(&mut self, service_arn: String, event_tx: EventSender) {
        // Extract service name from ARN for display
        let service_name = service_arn
            .split('/')
            .next_back()
            .unwrap_or(&service_arn)
            .to_string();

        self.services.ecs.selected_service_arn = Some(service_arn);
        self.services.ecs.selected_service_name = Some(service_name.clone());
        self.services.ecs.view_mode = EcsViewMode::Tasks;
        self.services.ecs.tasks.clear();
        self.services.ecs.list_state.select(None);

        let Some(cluster_arn) = self.services.ecs.selected_cluster_arn.clone() else {
            return;
        };

        // Capture for closure
        let s_name = service_name.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_TASKS,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                match ecs_client
                    .list_tasks(&cluster_arn, Some(&s_name))
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
        );
    }

    pub fn handle_ecs_view_task_definition(
        &mut self,
        task_definition_arn: String,
        event_tx: EventSender,
    ) {
        self.services.ecs.view_mode = EcsViewMode::TaskDefinition;
        self.services.ecs.current_task_definition = None;
        self.services.ecs.detail_scroll_offset = 0;

        let arn = task_definition_arn.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_TASK_DEFINITION,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                match ecs_client
                    .describe_task_definition(&arn)
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
            }
        );
    }

    pub fn handle_ecs_back_to_clusters(&mut self) {
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

    pub fn handle_ecs_back_to_services(&mut self) {
        self.services.ecs.view_mode = EcsViewMode::Services;
        self.services.ecs.clear_tasks();

        // Restore selection on services list
        if !self.services.ecs.services.is_empty()
            && self.services.ecs.list_state.selected().is_none()
        {
            self.services.ecs.list_state.select(Some(0));
        }
    }

    pub fn handle_ecs_back_to_tasks(&mut self) {
        self.services.ecs.view_mode = EcsViewMode::Tasks;
        self.services.ecs.current_task_definition = None;

        // Restore selection on tasks list
        if !self.services.ecs.tasks.is_empty() && self.services.ecs.list_state.selected().is_none()
        {
            self.services.ecs.list_state.select(Some(0));
        }
    }
}
