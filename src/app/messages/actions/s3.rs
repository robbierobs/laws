//! S3-specific actions

/// S3-specific actions
#[derive(Debug, Clone)]
pub enum S3Action {
    LoadObjects(String),
    LoadBucketDetails(String),
    /// Create a new S3 bucket
    CreateBucket(String),
    /// Delete an S3 bucket (must be empty)
    DeleteBucket(String),
    DeleteObject {
        bucket: String,
        key: String,
    },
    /// Download object to ~/Downloads directory
    DownloadObject {
        bucket: String,
        key: String,
    },
    /// Open object (download to temp dir and display in terminal/viewer)
    OpenObject {
        bucket: String,
        key: String,
    },
    /// Edit object in $EDITOR and upload changes
    EditObject {
        bucket: String,
        key: String,
    },
    LeaveBucket,
}
