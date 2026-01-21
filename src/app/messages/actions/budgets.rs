//! Budgets action enums

/// Actions for the Budgets service
#[derive(Debug, Clone)]
pub enum BudgetsAction {
    /// Load notifications for a specific budget
    LoadNotifications(String),
    /// Leave notifications view and return to budgets list
    LeaveNotifications,
}

impl crate::app::messages::ConfirmableAction for BudgetsAction {
    fn confirmation_description(&self) -> String {
        // No destructive actions in budgets (read-only for now)
        match self {
            Self::LoadNotifications(name) => format!("Load notifications for budget: {}", name),
            Self::LeaveNotifications => "Return to budgets list".to_string(),
        }
    }
}
