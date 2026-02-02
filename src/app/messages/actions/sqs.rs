//! SQS-specific actions

use super::super::confirmable::ConfirmableAction;

/// SQS-specific actions
#[derive(Debug, Clone)]
pub enum SqsAction {
    PurgeQueue(String),
    DeleteQueue(String),
}

impl ConfirmableAction for SqsAction {
    fn confirmation_description(&self) -> String {
        match self {
            Self::PurgeQueue(name) => format!("Purge all messages from queue {}", name),
            Self::DeleteQueue(name) => format!("Delete queue {}", name),
        }
    }
}
