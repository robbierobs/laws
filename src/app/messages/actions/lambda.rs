//! Lambda-specific actions

use super::super::confirmable::ConfirmableAction;

/// Lambda-specific actions
#[derive(Debug, Clone)]
pub enum LambdaAction {
    InvokeFunction(String),
    DeleteFunction(String),
    LoadFunctionDetails(String),
}

impl ConfirmableAction for LambdaAction {
    fn confirmation_description(&self) -> String {
        match self {
            Self::InvokeFunction(name) => format!("Invoke Lambda function {}", name),
            Self::DeleteFunction(name) => format!("Delete Lambda function {}", name),
            _ => "Lambda operation".to_string(),
        }
    }
}
