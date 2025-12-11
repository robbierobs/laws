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
    pub async fn list_attached_user_policies(&self, user_name: &str) -> anyhow::Result<Vec<IamPolicy>> {
        let response = self.client
            .list_attached_user_policies()
            .user_name(user_name)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list attached user policies: {}", e))?;

        let mut policies = Vec::new();
        for policy in response.attached_policies() {
            policies.push(IamPolicy {
                policy_name: policy.policy_name().unwrap_or_default().to_string(),
                policy_id: None, // AttachedPolicy doesn't have ID
                arn: Some(policy.policy_arn().unwrap_or_default().to_string()),
                create_date: None,
                update_date: None,
                attachment_count: None,
                is_attachable: true,
            });
        }
        Ok(policies)
    }

    pub async fn list_attached_role_policies(&self, role_name: &str) -> anyhow::Result<Vec<IamPolicy>> {
        let response = self.client
            .list_attached_role_policies()
            .role_name(role_name)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list attached role policies: {}", e))?;

        let mut policies = Vec::new();
        for policy in response.attached_policies() {
            policies.push(IamPolicy {
                policy_name: policy.policy_name().unwrap_or_default().to_string(),
                policy_id: None,
                arn: Some(policy.policy_arn().unwrap_or_default().to_string()),
                create_date: None,
                update_date: None,
                attachment_count: None,
                is_attachable: true,
            });
        }
        Ok(policies)
    }

    pub async fn get_policy_version(&self, policy_arn: &str) -> anyhow::Result<String> {
        // First get the policy to find the default version
        let policy = self.client
            .get_policy()
            .policy_arn(policy_arn)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get policy: {}", e))?;

        let version_id = policy.policy()
            .and_then(|p| p.default_version_id())
            .ok_or_else(|| anyhow::anyhow!("No default version ID found"))?;

        let version = self.client
            .get_policy_version()
            .policy_arn(policy_arn)
            .version_id(version_id)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get policy version: {}", e))?;

        let document = version.policy_version()
            .and_then(|v| v.document())
            .map(|d| d.to_string()) // This might be URL encoded
            .unwrap_or_default();

        // Try to URL decode if needed, but for now just return as is
        let decoded = urlencoding::decode(&document).map(|s| s.to_string()).unwrap_or(document);
        
        // Try to pretty print JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&decoded) {
            Ok(serde_json::to_string_pretty(&json).unwrap_or(decoded))
        } else {
            Ok(decoded)
        }
    }
}

impl crate::aws::traits::AwsService<IamUser> for IamService {
    fn list<'a>(&'a self) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<IamUser>>> + Send + 'a>> {
        Box::pin(self.list_users())
    }
}
