use crate::error::AppResult;
use crate::models::vpc::{SecurityGroup, Subnet, Vpc};
use crate::utils::error::format_sdk_error;
use aws_sdk_ec2::Client;

// Use macro to generate struct and constructor
crate::aws_service_struct!(VpcService, Client);

impl VpcService {

    pub async fn list_vpcs(&self) -> AppResult<Vec<Vpc>> {
        let mut vpcs = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.describe_vpcs();
            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format_sdk_error("VPC", "describe_vpcs", "all", e))?;

            vpcs.extend(response.vpcs().iter().map(Vpc::from_aws));

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(vpcs)
    }

    pub async fn list_subnets(&self, vpc_id: Option<&str>) -> AppResult<Vec<Subnet>> {
        let mut subnets = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.describe_subnets();

            if let Some(id) = vpc_id {
                request = request.filters(
                    aws_sdk_ec2::types::Filter::builder()
                        .name("vpc-id")
                        .values(id)
                        .build(),
                );
            }

            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format_sdk_error("VPC", "describe_subnets", vpc_id.unwrap_or("all"), e))?;

            subnets.extend(
                response
                    .subnets()
                    .iter()
                    .map(Subnet::from_aws),
            );

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(subnets)
    }

    pub async fn list_security_groups(
        &self,
        vpc_id: Option<&str>,
    ) -> AppResult<Vec<SecurityGroup>> {
        let mut sgs = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.describe_security_groups();

            if let Some(id) = vpc_id {
                request = request.filters(
                    aws_sdk_ec2::types::Filter::builder()
                        .name("vpc-id")
                        .values(id)
                        .build(),
                );
            }

            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request.send().await.map_err(|e| {
                format_sdk_error(
                    "VPC",
                    "describe_security_groups",
                    vpc_id.unwrap_or("all"),
                    e,
                )
            })?;

            sgs.extend(
                response
                    .security_groups()
                    .iter()
                    .map(SecurityGroup::from_aws),
            );

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(sgs)
    }
    pub async fn delete_security_group(&self, group_id: &str) -> AppResult<()> {
        self.client
            .delete_security_group()
            .group_id(group_id)
            .send()
            .await
            .map_err(|e| format_sdk_error("VPC", "delete_security_group", group_id, e))?;
        Ok(())
    }
}

impl crate::aws::traits::AwsService<Vpc> for VpcService {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<Vpc>>> + Send + 'a>>
    {
        Box::pin(self.list_vpcs())
    }
}
