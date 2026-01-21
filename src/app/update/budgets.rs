//! Budgets action handler

use crate::app::messages::BudgetsAction;
use crate::app::{App, BudgetsViewMode};
use crate::aws::budgets::BudgetsService;
use crate::event::{AwsEvent, Event};

impl App {
    pub fn handle_budgets_action(
        &mut self,
        action: BudgetsAction,
        event_tx: crate::app::EventSender,
    ) {
        match action {
            BudgetsAction::LoadNotifications(budget_name) => {
                self.load_budget_notifications(&budget_name, event_tx);
            }
            BudgetsAction::LeaveNotifications => {
                self.services.budgets.notifications.clear();
                self.services.budgets.selected_budget = None;
                self.services.budgets.view_mode = BudgetsViewMode::Budgets;
                self.services.budgets.list_state.select(Some(0));
            }
        }
    }

    fn load_budget_notifications(&mut self, budget_name: &str, event_tx: crate::app::EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };
        let Some(account_id) = self.services.budgets.account_id.clone() else {
            return;
        };

        self.loading = true;
        let budgets_client = clients.budgets.clone();
        let budget_name = budget_name.to_string();

        let handle = tokio::spawn(async move {
            let service = BudgetsService::new(budgets_client);
            match service.describe_notifications_for_budget(&account_id, &budget_name).await {
                Ok(notifications) => {
                    event_tx
                        .send(Event::Aws(Box::new(AwsEvent::BudgetNotificationsLoaded {
                            budget_name,
                            notifications,
                        })))
                        .await
                        .ok();
                }
                Err(e) => {
                    event_tx
                        .send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                        .await
                        .ok();
                }
            }
        });

        self.tasks.spawn("budgets:notifications", handle);
    }
}
