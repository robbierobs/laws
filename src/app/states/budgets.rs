//! State for AWS Budgets service

use ratatui::widgets::TableState;
use crate::models::budgets::{Budget, BudgetNotification};
use crate::models::billing::BillingView;
use crate::app::{BudgetsViewMode, InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::view_mode::ViewMode;
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::aws::budgets::BudgetsService;
use crate::aws::billing::BillingService;
use crate::event::{AwsEvent, Event};
use crossterm::event::{KeyCode, KeyEvent};

/// State for AWS Budgets service
pub struct BudgetsState {
    pub budgets: Vec<Budget>,
    pub notifications: Vec<BudgetNotification>,
    pub billing_views: Vec<BillingView>,
    pub selected_budget: Option<String>,
    pub view_mode: BudgetsViewMode,
    pub list_state: TableState,
    /// AWS Account ID (needed for Budgets API)
    pub account_id: Option<String>,
}

impl Default for BudgetsState {
    fn default() -> Self {
        Self {
            budgets: Vec::new(),
            notifications: Vec::new(),
            billing_views: Vec::new(),
            selected_budget: None,
            view_mode: BudgetsViewMode::default(),
            list_state: TableState::default(),
            account_id: None,
        }
    }
}

impl BudgetsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected budget
    pub fn selected_budget(&self) -> Option<&Budget> {
        if self.view_mode == BudgetsViewMode::Budgets {
            self.list_state.selected().and_then(|i| self.budgets.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected notification
    pub fn selected_notification(&self) -> Option<&BudgetNotification> {
        if self.view_mode == BudgetsViewMode::Notifications {
            self.list_state.selected().and_then(|i| self.notifications.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected billing view
    pub fn selected_billing_view(&self) -> Option<&BillingView> {
        if self.view_mode == BudgetsViewMode::BillingViews {
            self.list_state.selected().and_then(|i| self.billing_views.get(i))
        } else {
            None
        }
    }

    /// Check if we're currently viewing notifications for a budget
    pub fn is_viewing_notifications(&self) -> bool {
        self.view_mode == BudgetsViewMode::Notifications && self.selected_budget.is_some()
    }
}

impl crate::app::global_search::Searchable for BudgetsState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        self.budgets
            .iter()
            .map(|b| {
                let mut result = SearchResult::new(Service::Budgets, "Budget", &b.budget_name);
                if let Some(pct) = b.usage_percentage() {
                    result = result.with_secondary(format!("{:.0}% used", pct));
                }
                result
            })
            .collect()
    }
}

impl crate::app::global_search::AutoSelectable for BudgetsState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        self.view_mode = BudgetsViewMode::Budgets;
        if let Some(idx) = self.budgets.iter().position(|b| b.budget_name == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

impl ServiceInputHandler for BudgetsState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        let len = match self.view_mode {
            BudgetsViewMode::Budgets => self.budgets.len(),
            BudgetsViewMode::Notifications => self.notifications.len(),
            BudgetsViewMode::BillingViews => self.billing_views.len(),
        };
        
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter => {
                if self.view_mode == BudgetsViewMode::Budgets {
                    if let Some(budget) = self.selected_budget() {
                        return InputResult::Message(Message::budgets_load_notifications(
                            budget.budget_name.clone(),
                        ));
                    }
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                if self.is_viewing_notifications() {
                    return InputResult::Message(Message::budgets_leave_notifications());
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
            BudgetsViewMode::Budgets => self.selected_budget().map(|b| b.budget_name.clone()),
            BudgetsViewMode::Notifications => {
                self.selected_notification().map(|n| n.threshold_display())
            }
            BudgetsViewMode::BillingViews => {
                self.selected_billing_view().map(|v| v.arn.clone())
            }
        }
    }
}

impl ServiceInternal for BudgetsState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let budgets_client = clients.budgets.clone();
        let billing_client = clients.billing.clone();
        let account_id = self.account_id.clone();
        
        let handle = tokio::spawn(async move {
            // Load billing views first (doesn't need account_id)
            let billing_service = BillingService::new(billing_client);
            if let Ok(views) = billing_service.list_billing_views().await {
                tx.send(Event::Aws(Box::new(AwsEvent::BillingViewsLoaded(views))))
                    .await
                    .ok();
            }
            
            // Load budgets (requires account_id)
            if let Some(account_id) = account_id {
                let budgets_service = BudgetsService::new(budgets_client);
                match budgets_service.list_budgets(&account_id).await {
                    Ok(budgets) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::BudgetsLoaded(budgets))))
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
            }
        });
        tasks.spawn(task_keys::BUDGETS_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.budgets.clear();
        self.notifications.clear();
        self.billing_views.clear();
        self.selected_budget = None;
        self.view_mode = BudgetsViewMode::Budgets;
        self.list_state.select(Some(0));
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() {
            let has_items = match self.view_mode {
                BudgetsViewMode::Budgets => !self.budgets.is_empty(),
                BudgetsViewMode::Notifications => !self.notifications.is_empty(),
                BudgetsViewMode::BillingViews => !self.billing_views.is_empty(),
            };
            if has_items {
                self.list_state.select(Some(0));
            }
        }
    }

    fn can_cycle_view(&self) -> bool {
        // Can cycle between Budgets and BillingViews tabs
        self.view_mode.is_main_tab()
    }
}
