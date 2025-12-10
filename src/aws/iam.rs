use crate::models::iam::{IamUser, IamRole, IamPolicy};
use aws_sdk_iam::Client;

pub struct IamService {
    client: Client,
}

impl IamService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_users(&self) -> anyhow::Result<Vec<IamUser>> {
        let mut users = Vec::new();
        let mut marker: Option<String> = None;

        loop {
            let mut request = self.client.list_users();
            if let Some(m) = marker {
                request = request.marker(m);
            }

            let response = request
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to list IAM users: {}", e))?;

            for user in response.users() {
                users.push(IamUser::from_aws(user));
            }

            marker = response.marker().map(|s| s.to_string());
            if marker.is_none() || !response.is_truncated() {
                break;
            }
        }

        Ok(users)
    }

    pub async fn list_roles(&self) -> anyhow::Result<Vec<IamRole>> {
        let mut roles = Vec::new();
        let mut marker: Option<String> = None;

        loop {
            let mut request = self.client.list_roles();
            if let Some(m) = marker {
                request = request.marker(m);
            }

            let response = request
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to list IAM roles: {}", e))?;

            for role in response.roles() {
                roles.push(IamRole::from_aws(role));
            }

            marker = response.marker().map(|s| s.to_string());
            if marker.is_none() || !response.is_truncated() {
                break;
            }
        }

        Ok(roles)
    }

    pub async fn list_policies(&self) -> anyhow::Result<Vec<IamPolicy>> {
        let mut policies = Vec::new();
        let mut marker: Option<String> = None;

        loop {
            let mut request = self.client.list_policies().scope(aws_sdk_iam::types::PolicyScopeType::Local);
            if let Some(m) = marker {
                request = request.marker(m);
            }

            let response = request
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to list IAM policies: {}", e))?;

            for policy in response.policies() {
                policies.push(IamPolicy::from_aws(policy));
            }

            marker = response.marker().map(|s| s.to_string());
            if marker.is_none() || !response.is_truncated() {
                break;
            }
        }

        Ok(policies)
    }
}
