use ratatui::widgets::TableState;
use crate::models::backup::{BackupJob, BackupPlan, BackupVault, RecoveryPoint};
use crate::app::{BackupViewMode, InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::aws::backup::BackupService;
use crate::event::{AwsEvent, Event};
use crossterm::event::{KeyCode, KeyEvent};

/// State for AWS Backup service
#[derive(Default)]
pub struct BackupState {
    pub vaults: Vec<BackupVault>,
    pub plans: Vec<BackupPlan>,
    pub jobs: Vec<BackupJob>,
    pub recovery_points: Vec<RecoveryPoint>,
    pub list_state: TableState,
    pub view_mode: BackupViewMode,
}

impl BackupState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected vault, if any
    pub fn selected_vault(&self) -> Option<&BackupVault> {
        if self.view_mode == BackupViewMode::Vaults {
            self.list_state.selected().and_then(|i| self.vaults.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected plan, if any
    pub fn selected_plan(&self) -> Option<&BackupPlan> {
        if self.view_mode == BackupViewMode::Plans {
            self.list_state.selected().and_then(|i| self.plans.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected job, if any
    pub fn selected_job(&self) -> Option<&BackupJob> {
        if self.view_mode == BackupViewMode::Jobs {
            self.list_state.selected().and_then(|i| self.jobs.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected recovery point, if any
    pub fn selected_recovery_point(&self) -> Option<&RecoveryPoint> {
        if self.view_mode == BackupViewMode::RecoveryPoints {
            self.list_state.selected().and_then(|i| self.recovery_points.get(i))
        } else {
            None
        }
    }
}

impl crate::app::global_search::Searchable for BackupState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();
        // Backup Vaults
        for vault in &self.vaults {
            results.push(SearchResult::new(
                Service::Backup,
                "Backup Vault",
                &vault.backup_vault_name,
            ));
        }
        results
    }
}

impl crate::app::global_search::AutoSelectable for BackupState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        self.view_mode = BackupViewMode::Vaults;
        if let Some(idx) = self.vaults.iter().position(|v| v.backup_vault_name == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

impl ServiceInputHandler for BackupState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        let len = match self.view_mode {
            BackupViewMode::Vaults => self.vaults.len(),
            BackupViewMode::Plans => self.plans.len(),
            BackupViewMode::Jobs => self.jobs.len(),
            BackupViewMode::RecoveryPoints => self.recovery_points.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter => {
                if self.view_mode == BackupViewMode::Vaults {
                    if let Some(vault) = self.selected_vault() {
                        return InputResult::Message(Message::backup_load_recovery_points(
                            vault.backup_vault_name.clone(),
                        ));
                    }
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                if self.view_mode == BackupViewMode::RecoveryPoints {
                    // Go back to vaults
                    return InputResult::Message(Message::backup_leave_vault());
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }

    fn get_copiable_text(&self) -> Option<String> {
        match self.view_mode {
            BackupViewMode::Vaults => self.selected_vault().map(|v| v.backup_vault_name.clone()),
            BackupViewMode::Plans => self.selected_plan().map(|p| p.backup_plan_id.clone()),
            BackupViewMode::Jobs => self.selected_job().map(|j| j.backup_job_id.clone()),
            BackupViewMode::RecoveryPoints => self.selected_recovery_point().map(|rp| rp.recovery_point_arn.clone()),
        }
    }
}

impl ServiceInternal for BackupState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.backup.clone();
        let handle = tokio::spawn(async move {
            let service = BackupService::new(client);
            match service.list_backup_vaults().await {
                Ok(vaults) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::BackupVaultsLoaded(vaults))))
                        .await
                        .ok();
                }
                Err(e) => {
                    if report_errors {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                            .await
                            .ok();
                    }
                }
            }
            if let Ok(plans) = service.list_backup_plans().await {
                tx.send(Event::Aws(Box::new(AwsEvent::BackupPlansLoaded(plans))))
                    .await
                    .ok();
            }
            if let Ok(jobs) = service.list_backup_jobs().await {
                tx.send(Event::Aws(Box::new(AwsEvent::BackupJobsLoaded(jobs))))
                    .await
                    .ok();
            }
        });
        tasks.spawn(task_keys::BACKUP_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.vaults.clear();
        self.plans.clear();
        self.jobs.clear();
        self.recovery_points.clear();
        self.view_mode = BackupViewMode::Vaults;
        self.list_state.select(Some(0));
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() {
            let has_items = match self.view_mode {
                BackupViewMode::Vaults => !self.vaults.is_empty(),
                BackupViewMode::Plans => !self.plans.is_empty(),
                BackupViewMode::Jobs => !self.jobs.is_empty(),
                BackupViewMode::RecoveryPoints => !self.recovery_points.is_empty(),
            };
            if has_items {
                self.list_state.select(Some(0));
            }
        }
    }
}
