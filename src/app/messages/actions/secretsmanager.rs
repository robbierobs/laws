//! SecretsManager-specific actions

/// SecretsManager-specific actions
#[derive(Debug, Clone)]
pub enum SecretsManagerAction {
    GetSecretValue(String),
    CloseSecretValue,
    DeleteSecret(String),
}
