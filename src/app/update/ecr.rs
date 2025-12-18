use crate::app::{
    messages::{EcrAction, EcrViewMode},
    App,
};
use crate::app::task_manager::task_keys;
use crate::event::{AwsEvent, Event};

impl App {
    pub fn handle_ecr_action(
        &mut self,
        action: EcrAction,
        event_tx: crate::app::EventSender,
    ) {
        match action {
            EcrAction::LoadImages(repo_name) => {
                self.services.ecr.selected_repo_name = Some(repo_name.clone());
                self.services.ecr.view_mode = EcrViewMode::Images;
                self.services.ecr.images.clear();
                self.services.ecr.list_state.select(None);

                if let Some(clients) = &self.aws_clients {
                    let client = clients.ecr.clone();
                    let tx = event_tx.clone();
                    let repo_name_clone = repo_name.clone();
                    
                    let handle = tokio::spawn(async move {
                        let ecr_service = crate::aws::ecr::EcrService::new(client);
                        match ecr_service.describe_images(&repo_name_clone).await {
                            Ok(images) => {
                                tx.send(Event::Aws(Box::new(AwsEvent::EcrImagesLoaded(images))))
                                    .await
                                    .ok();
                            }
                            Err(e) => {
                                tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                                    .await
                                    .ok();
                            }
                        }
                    });
                    self.tasks.spawn(task_keys::ECR_IMAGES, handle);
                }
            }
            EcrAction::BackToRepositories => {
                self.services.ecr.view_mode = EcrViewMode::Repositories;
                self.services.ecr.selected_repo_name = None;
                self.services.ecr.images.clear();
                self.services.ecr.list_state.select(None);
            }
        }
    }
}
