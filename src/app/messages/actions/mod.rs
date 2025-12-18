//! Per-service action enums
//!
//! Each service has its own action enum defining the operations that can be performed.

mod ec2;
mod s3;
mod rds;
mod dynamodb;
mod lambda;
mod vpc;
mod iam;
mod backup;
mod cloudtrail;
mod secretsmanager;
mod ecs;
mod ecr;

pub use ec2::Ec2Action;
pub use s3::S3Action;
pub use rds::RdsAction;
pub use dynamodb::DynamoDbAction;
pub use lambda::LambdaAction;
pub use vpc::VpcAction;
pub use iam::IamAction;
pub use backup::BackupAction;
pub use cloudtrail::CloudTrailAction;
pub use secretsmanager::SecretsManagerAction;
pub use ecs::EcsAction;
pub use ecr::EcrAction;
