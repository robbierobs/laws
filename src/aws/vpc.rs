use crate::models::vpc::{Vpc, Subnet, SecurityGroup};
use aws_sdk_ec2::Client;

pub struct VpcService {
    client: Client,
}

impl VpcService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_vpcs(&self) -> anyhow::Result<Vec<Vpc>> {
        let response = self.client
            .describe_vpcs()
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list VPCs: {}", e))?;

        let vpcs = response.vpcs()
            .iter()
            .map(|v| Vpc::from_aws(v))
            .collect();

        Ok(vpcs)
    }

    pub async fn list_subnets(&self, vpc_id: Option<&str>) -> anyhow::Result<Vec<Subnet>> {
        let mut request = self.client.describe_subnets();
        
        if let Some(id) = vpc_id {
            request = request.filters(
                aws_sdk_ec2::types::Filter::builder()
                    .name("vpc-id")
                    .values(id)
                    .build()
            );
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list subnets: {}", e))?;

        let subnets = response.subnets()
            .iter()
            .map(|s| Subnet::from_aws(s))
            .collect();

        Ok(subnets)
    }

    pub async fn list_security_groups(&self, vpc_id: Option<&str>) -> anyhow::Result<Vec<SecurityGroup>> {
        let mut request = self.client.describe_security_groups();
        
        if let Some(id) = vpc_id {
            request = request.filters(
                aws_sdk_ec2::types::Filter::builder()
                    .name("vpc-id")
                    .values(id)
                    .build()
            );
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list security groups: {}", e))?;

        let sgs = response.security_groups()
            .iter()
            .map(|sg| SecurityGroup::from_aws(sg))
            .collect();

        Ok(sgs)
    }
}
