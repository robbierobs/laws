use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::Filterable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcrRepository {
    pub repository_name: String,
    pub repository_arn: Option<String>,
    pub repository_uri: Option<String>,
    pub created_at: Option<String>,
    pub image_tag_mutability: Option<String>,
}

impl EcrRepository {
    pub fn from_aws(repo: &aws_sdk_ecr::types::Repository) -> Self {
        Self {
            repository_name: repo.repository_name().unwrap_or_default().to_string(),
            repository_arn: repo.repository_arn().map(|s| s.to_string()),
            repository_uri: repo.repository_uri().map(|s| s.to_string()),
            created_at: repo.created_at().map(|d| d.to_string()),
            image_tag_mutability: repo.image_tag_mutability().map(|m| m.as_str().to_string()),
        }
    }
}

impl Filterable for EcrRepository {
    fn matches_filter(&self, filter: &str) -> bool {
        self.repository_name.to_lowercase().contains(filter)
            || self
                .repository_uri
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
    }
}

/// Image scan status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageScanStatus {
    pub status: Option<String>,
    pub description: Option<String>,
}

impl ImageScanStatus {
    pub fn from_aws(scan_status: &aws_sdk_ecr::types::ImageScanStatus) -> Self {
        Self {
            status: scan_status.status().map(|s| s.as_str().to_string()),
            description: scan_status.description().map(|s| s.to_string()),
        }
    }
}

/// Summary of image scan findings with severity counts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageScanFindingsSummary {
    pub scan_completed_at: Option<String>,
    pub vulnerability_source_updated_at: Option<String>,
    /// Severity counts: CRITICAL, HIGH, MEDIUM, LOW, INFORMATIONAL, UNDEFINED
    pub finding_severity_counts: HashMap<String, i32>,
}

impl ImageScanFindingsSummary {
    pub fn from_aws(summary: &aws_sdk_ecr::types::ImageScanFindingsSummary) -> Self {
        let counts = summary
            .finding_severity_counts()
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.as_str().to_string(), *v))
                    .collect()
            })
            .unwrap_or_default();

        Self {
            scan_completed_at: summary.image_scan_completed_at().map(|d| d.to_string()),
            vulnerability_source_updated_at: summary
                .vulnerability_source_updated_at()
                .map(|d| d.to_string()),
            finding_severity_counts: counts,
        }
    }

    /// Get total vulnerability count
    pub fn total_count(&self) -> i32 {
        self.finding_severity_counts.values().sum()
    }

    /// Get count for a specific severity (case-insensitive)
    pub fn get_count(&self, severity: &str) -> i32 {
        self.finding_severity_counts
            .get(&severity.to_uppercase())
            .copied()
            .unwrap_or(0)
    }

    /// Get critical + high count (most important)
    pub fn critical_high_count(&self) -> i32 {
        self.get_count("CRITICAL") + self.get_count("HIGH")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcrImage {
    pub image_digest: String,
    pub image_tags: Vec<String>,
    pub image_pushed_at: Option<String>,
    pub image_size_in_bytes: Option<i64>,
    pub image_uri: Option<String>,
    /// Scan status (if scanning enabled)
    pub scan_status: Option<ImageScanStatus>,
    /// Scan findings summary (if scan completed)
    pub scan_findings_summary: Option<ImageScanFindingsSummary>,
}

impl EcrImage {
    pub fn from_aws(image: &aws_sdk_ecr::types::ImageDetail) -> Self {
        Self {
            image_digest: image.image_digest().unwrap_or_default().to_string(),
            image_tags: image.image_tags().iter().map(|t| t.to_string()).collect(),
            image_pushed_at: image.image_pushed_at().map(|d| d.to_string()),
            image_size_in_bytes: image.image_size_in_bytes(),
            image_uri: None,
            scan_status: image.image_scan_status().map(ImageScanStatus::from_aws),
            scan_findings_summary: image
                .image_scan_findings_summary()
                .map(ImageScanFindingsSummary::from_aws),
        }
    }

    #[allow(dead_code)]
    pub fn main_tag(&self) -> String {
        self.image_tags
            .first()
            .cloned()
            .unwrap_or_else(|| "<untagged>".to_string())
    }

    /// Check if scan is complete
    pub fn is_scan_complete(&self) -> bool {
        self.scan_status
            .as_ref()
            .and_then(|s| s.status.as_ref())
            .map(|s| s == "COMPLETE")
            .unwrap_or(false)
    }

    /// Get a short scan status display string
    pub fn scan_status_display(&self) -> &'static str {
        match self.scan_status.as_ref().and_then(|s| s.status.as_deref()) {
            Some("COMPLETE") => "✓",
            Some("IN_PROGRESS") | Some("PENDING") => "⏳",
            Some("FAILED") => "✗",
            Some("UNSUPPORTED_IMAGE") => "N/A",
            Some(_) => "?",
            None => "-",
        }
    }
}

impl Filterable for EcrImage {
    fn matches_filter(&self, filter: &str) -> bool {
        self.image_digest.to_lowercase().contains(filter)
            || self
                .image_tags
                .iter()
                .any(|t| t.to_lowercase().contains(filter))
    }
}
