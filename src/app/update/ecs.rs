use crate::app::messages::{EcsAction, EcsViewMode};
use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};

impl App {
    pub async fn handle_ecs_action(&mut self, action: EcsAction, event_tx: EventSender) {
        match action {
            EcsAction::ViewServices(cluster_arn) => {
                self.services.ecs.selected_cluster_arn = Some(cluster_arn.clone());
                self.services.ecs.view_mode = EcsViewMode::Services;
                self.services.ecs.services.clear();
                self.services.ecs.list_state.select(None);

                // Trigger load services
                if let Some(clients) = &self.aws_clients {
                    let client = clients.ecs.clone();
                    let tx = event_tx.clone();
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
                    // We reuse a generic refresh key or define a specific one?
                    // Let's use generic one or define new one. Generic generic refresh is usually for main list.
                    // Let's us ecs:services:{cluster_arn} as key would be better but simple string is fine.
                    self.tasks.spawn("ecs:services", handle);
                }
            }
            EcsAction::BackToClusters => {
                self.services.ecs.view_mode = EcsViewMode::Clusters;
                self.services.ecs.selected_cluster_arn = None;
                self.services.ecs.services.clear();
            }
        }
    }
}
