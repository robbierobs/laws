use crate::app::task_manager::task_keys;
use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};
use crate::app::messages::BackupAction;
use crate::app::BackupViewMode;

impl App {
    pub fn handle_backup_action(&mut self, action: BackupAction, event_tx: EventSender) {
        match action {
            BackupAction::LoadRecoveryPoints(vault_name) => {
                self.handle_backup_load_recovery_points(vault_name, event_tx);
            }
            BackupAction::LeaveVault => {
                self.handle_backup_leave_vault();
            }
        }
    }

    pub fn handle_backup_load_recovery_points(&mut self, vault_name: String, event_tx: EventSender) {
        self.services.backup.view_mode = BackupViewMode::RecoveryPoints;
        self.services.backup.recovery_points.clear();
        self.services.backup.list_state.select(None);
        // We might want to store current vault name in state if needed, but for now we just load points.
        // Actually, if we want to display "Recovery Points for Vault X", we should probably store it.
        // But BackupState doesn't have `current_vault` field yet? 
        // Let's check `BackupState` again. 
        // It has `vaults: Vec<BackupVault>`.
        // It does NOT have `current_vault`. 
        // I should probably rely on `selected_vault()` but if the user changes tabs...
        // Wait, BackupViewMode::RecoveryPoints is a drill-down. 
        // Similar to S3 `current_bucket`.
        // I'll assume for now we just load and show.
        
        // Use generic task spawner
        let v_name = vault_name.clone();
        
        self.spawn_aws_task(
            event_tx,
            task_keys::BACKUP_REFRESH, // Use BACKUP_REFRESH
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
        // Restore selection if possible? 
        // Usually handled by state management automatically if index preserved.
    }
}
