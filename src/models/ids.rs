//! Newtype wrappers for AWS resource identifiers
//!
//! These types provide type safety to prevent accidentally mixing up
//! different kinds of resource IDs (e.g., passing an S3 bucket name
//! where an EC2 instance ID is expected).

use std::fmt;

// =========================================
// EC2 Resource IDs
// =========================================

/// EC2 Instance ID (e.g., "i-1234567890abcdef0")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ec2InstanceId(pub String);

impl Ec2InstanceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Ec2InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Ec2InstanceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Ec2InstanceId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// =========================================
// S3 Resource IDs
// =========================================

/// S3 Bucket Name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct S3BucketName(pub String);

impl S3BucketName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for S3BucketName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for S3BucketName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// S3 Object Key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct S3ObjectKey(pub String);

impl S3ObjectKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for S3ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for S3ObjectKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// =========================================
// RDS Resource IDs
// =========================================

/// RDS DB Instance Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RdsInstanceId(pub String);

impl RdsInstanceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RdsInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RdsInstanceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// =========================================
// DynamoDB Resource IDs
// =========================================

/// DynamoDB Table Name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DynamoDbTableName(pub String);

impl DynamoDbTableName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DynamoDbTableName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DynamoDbTableName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// =========================================
// Lambda Resource IDs
// =========================================

/// Lambda Function Name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LambdaFunctionName(pub String);

impl LambdaFunctionName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LambdaFunctionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for LambdaFunctionName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// =========================================
// VPC Resource IDs
// =========================================

/// VPC ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VpcId(pub String);

impl VpcId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VpcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for VpcId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Security Group ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecurityGroupId(pub String);

impl SecurityGroupId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SecurityGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SecurityGroupId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Subnet ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubnetId(pub String);

impl SubnetId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SubnetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SubnetId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// =========================================
// IAM Resource IDs
// =========================================

/// IAM User Name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IamUserName(pub String);

impl IamUserName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IamUserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for IamUserName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// IAM Role Name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IamRoleName(pub String);

impl IamRoleName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IamRoleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for IamRoleName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// IAM Policy ARN
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IamPolicyArn(pub String);

impl IamPolicyArn {
    pub fn new(arn: impl Into<String>) -> Self {
        Self(arn.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IamPolicyArn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for IamPolicyArn {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ec2_instance_id() {
        let id = Ec2InstanceId::new("i-1234567890abcdef0");
        assert_eq!(id.as_str(), "i-1234567890abcdef0");
        assert_eq!(format!("{}", id), "i-1234567890abcdef0");
    }

    #[test]
    fn test_s3_bucket_name() {
        let name = S3BucketName::from("my-bucket".to_string());
        assert_eq!(name.as_str(), "my-bucket");
    }

    #[test]
    fn test_from_string() {
        let id: Ec2InstanceId = "i-test".into();
        assert_eq!(id.0, "i-test");
    }
}
