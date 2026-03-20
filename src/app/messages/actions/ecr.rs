//! ECR-specific actions

use super::super::confirmable::ConfirmableAction;

/// Export format for scan findings
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
}

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
    /// Load scan findings for the selected image
    LoadScanFindings {
        repository_name: String,
        image_digest: String,
    },
    /// Export scan findings to file
    ExportScanFindings {
        format: ExportFormat,
    },
    /// Open the filter modal
    OpenFilterModal,
    #[allow(dead_code)] // TODO: Wire up ECR filter modal
    /// Close the filter modal without applying
    CloseFilterModal,
    /// Apply filters from modal
    ApplyFilters,
    /// Clear all filters
    ClearFilters,
}

impl ConfirmableAction for EcrAction {
    fn confirmation_description(&self) -> String {
        // ECR actions don't require confirmation
        "ECR operation".to_string()
    }
}
