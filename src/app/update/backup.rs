use crate::app::task_manager::task_keys;
use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};
use crate::app::messages::BackupAction;
use crate::app::BackupViewMode;
use crate::app::states::backup::BackupJobFilters;

impl App {
    pub fn handle_backup_action(&mut self, action: BackupAction, event_tx: EventSender) {
        match action {
            BackupAction::LoadRecoveryPoints(vault_name) => {
                self.handle_backup_load_recovery_points(vault_name, event_tx);
            }
            BackupAction::LeaveVault => {
                self.handle_backup_leave_vault();
            }
            BackupAction::OpenFilterModal => {
                self.services.backup.jobs_filter_modal.open();
                self.input_mode = crate::app::InputMode::BackupJobFilter;
            }
            BackupAction::CloseFilterModal => {
                self.services.backup.jobs_filter_modal.close();
                self.input_mode = crate::app::InputMode::Normal;
            }
            BackupAction::ApplyFilters => {
                // Update current filters from modal state
                self.services.backup.jobs_current_filters = BackupJobFilters::from_modal_state(
                    &self.services.backup.jobs_filter_modal
                );
                self.services.backup.jobs_filter_modal.close();
                self.input_mode = crate::app::InputMode::Normal;
                // Note: Filtering is applied client-side, no need to reload
            }
            BackupAction::ClearFilters => {
                self.services.backup.jobs_filter_modal.clear_all();
                self.services.backup.jobs_current_filters = BackupJobFilters::default();
            }
        }
    }

    pub fn handle_backup_load_recovery_points(&mut self, vault_name: String, event_tx: EventSender) {
        self.services.backup.view_mode = BackupViewMode::RecoveryPoints;
        self.services.backup.recovery_points.clear();
        self.services.backup.list_state.select(None);
        
        let v_name = vault_name.clone();
        
        self.spawn_aws_task(
            event_tx,
            task_keys::BACKUP_REFRESH,
            move |clients, tx| async move {
                let backup_client = crate::aws::backup::BackupService::new(clients.backup.clone());
                match backup_client.list_recovery_points(&v_name).await {
                    Ok(points) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::BackupRecoveryPointsLoaded(points))))
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
        );
    }

    pub fn handle_backup_leave_vault(&mut self) {
        self.services.backup.view_mode = BackupViewMode::Vaults;
        self.services.backup.recovery_points.clear();
    }
}
