//! Message and enum definitions for the application state machine

/// AWS Service types supported by the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    EC2,
    S3,
    RDS,
    DynamoDB,
    Lambda,
    VPC,
    IAM,
    Backup,
    CloudTrail,
}

impl Service {
    pub fn as_str(&self) -> &str {
        match self {
            Service::EC2 => "EC2",
            Service::S3 => "S3",
            Service::RDS => "RDS",
            Service::DynamoDB => "DynamoDB",
            Service::Lambda => "Lambda",
            Service::VPC => "VPC",
            Service::IAM => "IAM",
            Service::Backup => "Backup",
            Service::CloudTrail => "CloudTrail",
        }
    }

    pub fn iterator() -> impl Iterator<Item = Self> {
        [
            Self::EC2,
            Self::S3,
            Self::RDS,
            Self::DynamoDB,
            Self::Lambda,
            Self::VPC,
            Self::IAM,
            Self::Backup,
            Self::CloudTrail,
        ]
        .iter()
        .copied()
    }
}

/// Application messages for the update loop (Elm Architecture style)
pub enum Message {
    // Navigation
    NavigateToService(Service),

    // Actions
    RefreshData,
    ConfirmAction,
    CancelAction,
    StartInstance(String),
    StopInstance(String),
    RebootInstance(String),
    LoadS3Objects(String),
    LoadBucketDetails(String),
    DeleteS3Object(String, String), // bucket, key
    LeaveS3Bucket,
    // RDS actions
    StartRdsInstance(String),
    StopRdsInstance(String),
    RebootRdsInstance(String),

    // UI
    ToggleDetailPanel,
    CycleViewMode,
    NextView,
    PreviousView,
    ToggleActionLog,
    Quit,

    // VPC Specific
    DrillDownSecurityGroup,
    ExitSecurityGroupRules,
    ToggleSgRulesDirection, // Switch between inbound and outbound

    // IAM Specific
    DrillDownIamUser,
    DrillDownIamRole,
    DrillDownIamPolicy,
    ExitIamDrillDown,

    // DynamoDB Specific
    DrillDownDynamoDbTable,
    ExitDynamoDbDrillDown,
    LoadDynamoDbItems(String), // table_name
    DeleteDynamoDbItem(String, std::collections::HashMap<String, String>), // table_name, key attributes
}

/// Which pane has focus
pub enum Focus {
    Sidebar,
    Main,
}

/// Current input mode
#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Filtering,
}
