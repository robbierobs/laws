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

/// View mode for VPC service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum VpcViewMode {
    #[default]
    Vpcs = 0,
    Subnets = 1,
    SecurityGroups = 2,
    SecurityGroupRules = 3,
}

impl VpcViewMode {
    /// Get the next view mode (wraps to first after last)
    pub fn next(self) -> Self {
        match self {
            Self::Vpcs => Self::Subnets,
            Self::Subnets => Self::SecurityGroups,
            Self::SecurityGroups => Self::Vpcs, // Wrap around, skip rules
            Self::SecurityGroupRules => Self::SecurityGroupRules, // Stay in rules view
        }
    }

    /// Get the previous view mode (wraps to last before first)
    pub fn previous(self) -> Self {
        match self {
            Self::Vpcs => Self::SecurityGroups,
            Self::Subnets => Self::Vpcs,
            Self::SecurityGroups => Self::Subnets,
            Self::SecurityGroupRules => Self::SecurityGroupRules, // Stay in rules view
        }
    }

    /// Convert to index for tab display
    pub fn to_index(self) -> usize {
        self as usize
    }
}

/// View mode for IAM service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum IamViewMode {
    #[default]
    Users = 0,
    Roles = 1,
    Policies = 2,
    UserAttachedPolicies = 3,
    RoleAttachedPolicies = 4,
    PolicyDocument = 5,
}

impl IamViewMode {
    /// Get the next view mode (wraps to first after last, for main tabs only)
    pub fn next(self) -> Self {
        match self {
            Self::Users => Self::Roles,
            Self::Roles => Self::Policies,
            Self::Policies => Self::Users, // Wrap around
            _ => self, // Drill-down views don't cycle
        }
    }

    /// Get the previous view mode (wraps to last before first, for main tabs only)
    pub fn previous(self) -> Self {
        match self {
            Self::Users => Self::Policies,
            Self::Roles => Self::Users,
            Self::Policies => Self::Roles,
            _ => self, // Drill-down views don't cycle
        }
    }

    /// Convert to index for tab display
    pub fn to_index(self) -> usize {
        self as usize
    }

    /// Check if this is a main tab view (not a drill-down)
    pub fn is_main_tab(self) -> bool {
        matches!(self, Self::Users | Self::Roles | Self::Policies)
    }
}

/// View mode for Backup service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BackupViewMode {
    #[default]
    Vaults = 0,
    Plans = 1,
    Jobs = 2,
}

impl BackupViewMode {
    /// Get the next view mode (wraps to first after last)
    pub fn next(self) -> Self {
        match self {
            Self::Vaults => Self::Plans,
            Self::Plans => Self::Jobs,
            Self::Jobs => Self::Vaults,
        }
    }

    /// Get the previous view mode (wraps to last before first)
    pub fn previous(self) -> Self {
        match self {
            Self::Vaults => Self::Jobs,
            Self::Plans => Self::Vaults,
            Self::Jobs => Self::Plans,
        }
    }

    /// Convert to index for tab display
    pub fn to_index(self) -> usize {
        self as usize
    }
}

/// View mode for CloudTrail service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum CloudTrailViewMode {
    #[default]
    Trails = 0,
    Events = 1,
}

impl CloudTrailViewMode {
    /// Get the next view mode (wraps to first after last)
    pub fn next(self) -> Self {
        match self {
            Self::Trails => Self::Events,
            Self::Events => Self::Trails,
        }
    }

    /// Get the previous view mode (wraps to last before first)
    pub fn previous(self) -> Self {
        self.next() // Same as next for 2 items
    }

    /// Convert to index for tab display
    pub fn to_index(self) -> usize {
        self as usize
    }
}

/// View mode for DynamoDB service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DynamoDbViewMode {
    #[default]
    Tables = 0,
    Items = 1,
}

impl DynamoDbViewMode {
    /// Convert to index for tab display
    pub fn to_index(self) -> usize {
        self as usize
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
