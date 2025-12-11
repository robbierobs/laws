pub mod ec2;
pub mod s3;
pub mod rds;
pub mod dynamodb;
pub mod lambda;
pub mod vpc;
pub mod iam;
pub mod backup;
pub mod cloudtrail;
pub mod ids;

pub trait Filterable {
    fn matches_filter(&self, filter: &str) -> bool;
}
