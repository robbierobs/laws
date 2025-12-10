use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamUser {
    pub user_name: String,
    pub user_id: String,
    pub arn: Option<String>,
    pub create_date: Option<String>,
    pub password_last_used: Option<String>,
    pub path: Option<String>,
}

impl IamUser {
    pub fn from_aws(user: &aws_sdk_iam::types::User) -> Self {
        Self {
            user_name: user.user_name().to_string(),
            user_id: user.user_id().to_string(),
            arn: Some(user.arn().to_string()),
            create_date: Some(user.create_date().to_string()),
            password_last_used: user.password_last_used().map(|d| d.to_string()),
            path: Some(user.path().to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamRole {
    pub role_name: String,
    pub role_id: String,
    pub arn: Option<String>,
    pub create_date: Option<String>,
    pub description: Option<String>,
    pub max_session_duration: Option<i32>,
    pub path: Option<String>,
}

impl IamRole {
    pub fn from_aws(role: &aws_sdk_iam::types::Role) -> Self {
        Self {
            role_name: role.role_name().to_string(),
            role_id: role.role_id().to_string(),
            arn: Some(role.arn().to_string()),
            create_date: Some(role.create_date().to_string()),
            description: role.description().map(|s| s.to_string()),
            max_session_duration: role.max_session_duration(),
            path: Some(role.path().to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamPolicy {
    pub policy_name: String,
    pub policy_id: Option<String>,
    pub arn: Option<String>,
    pub create_date: Option<String>,
    pub update_date: Option<String>,
    pub attachment_count: Option<i32>,
    pub is_attachable: bool,
}

impl IamPolicy {
    pub fn from_aws(policy: &aws_sdk_iam::types::Policy) -> Self {
        Self {
            policy_name: policy.policy_name().unwrap_or_default().to_string(),
            policy_id: policy.policy_id().map(|s| s.to_string()),
            arn: policy.arn().map(|s| s.to_string()),
            create_date: policy.create_date().map(|d| d.to_string()),
            update_date: policy.update_date().map(|d| d.to_string()),
            attachment_count: policy.attachment_count(),
            is_attachable: policy.is_attachable(),
        }
    }
}
