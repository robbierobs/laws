//! ECR-specific actions

use super::super::confirmable::ConfirmableAction;

/// ECR-specific actions
#[derive(Debug, Clone)]
pub enum EcrAction {
    LoadImages(String),
    BackToRepositories,
    /// Pull an image using docker login + docker pull
    /// Contains (repository_uri, image_tag_or_digest)
    PullImage {
        repository_uri: String,
        image_tag: String,
    },
    /// Load more images (pagination)
    LoadMoreImages,
    /// Open the filter modal
    OpenFilterModal,
    /// Close the filter modal without applying
    CloseFilterModal,
    /// Apply filters from modal
    ApplyFilters,
}

impl ConfirmableAction for EcrAction {
    fn confirmation_description(&self) -> String {
        // ECR actions don't require confirmation
        "ECR operation".to_string()
    }
}
