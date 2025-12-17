//! ECR-specific actions

/// ECR-specific actions
#[derive(Debug, Clone)]
pub enum EcrAction {
    LoadImages(String),
    BackToRepositories,
}
