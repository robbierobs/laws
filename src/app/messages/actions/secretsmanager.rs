//! SecretsManager-specific actions

use super::super::confirmable::ConfirmableAction;

/// SecretsManager-specific actions
#[derive(Debug, Clone)]
pub enum SecretsManagerAction {
    GetSecretValue(String),
    CloseSecretValue,
    DeleteSecret(String),
}

impl ConfirmableAction for SecretsManagerAction {
    fn confirmation_description(&self) -> String {
        match self {
            Self::DeleteSecret(arn) => format!("Delete Secret {}", arn),
            _ => "Secrets Manager operation".to_string(),
        }
    }
}
