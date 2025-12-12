use crate::app::task_manager::task_keys;
use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};

impl App {
    pub async fn handle_ecs_stop_task(
        &mut self,
        cluster_arn: String,
        task_arn: String,
        event_tx: EventSender,
    ) {
        let cluster = cluster_arn.clone();
        let task = task_arn.clone();
        let service_name = self.services.ecs.selected_service_name.clone();
        let task_id = task_arn.split('/').last().unwrap_or(&task_arn).to_string();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_ACTION,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                match ecs_client
                    .stop_task(&cluster, &task, "Stopped by LazyAWS user")
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
                            .list_tasks(&cluster, service_name.as_deref())
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
            }
        );
    }
}
