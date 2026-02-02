//! Service enum definition

// AWS service names use capitalized acronyms (EC2, S3, RDS, VPC, IAM, ECS, ECR)
#![allow(clippy::upper_case_acronyms)]

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
    SecretsManager,
    ECS,
    ECR,
    Budgets,
    SQS,
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
            Service::SecretsManager => "SecretsManager",
            Service::ECS => "ECS",
            Service::ECR => "ECR",
            Service::Budgets => "Budgets",
            Service::SQS => "SQS",
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
            Self::SecretsManager,
            Self::ECS,
            Self::ECR,
            Self::Budgets,
            Self::SQS,
        ]
        .iter()
        .copied()
    }
}
