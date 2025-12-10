use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vpc {
    pub vpc_id: String,
    pub cidr_block: Option<String>,
    pub state: String,
    pub is_default: bool,
    pub name: Option<String>,
    pub owner_id: Option<String>,
    pub instance_tenancy: Option<String>,
    pub dhcp_options_id: Option<String>,
}

impl Vpc {
    pub fn from_aws(vpc: &aws_sdk_ec2::types::Vpc) -> Self {
        let name = vpc.tags()
            .iter()
            .find(|t| t.key() == Some("Name"))
            .and_then(|t| t.value())
            .map(|s| s.to_string());

        Self {
            vpc_id: vpc.vpc_id().unwrap_or_default().to_string(),
            cidr_block: vpc.cidr_block().map(|s| s.to_string()),
            state: vpc.state().map(|s| s.as_str().to_string()).unwrap_or_default(),
            is_default: vpc.is_default().unwrap_or(false),
            name,
            owner_id: vpc.owner_id().map(|s| s.to_string()),
            instance_tenancy: vpc.instance_tenancy().map(|t| t.as_str().to_string()),
            dhcp_options_id: vpc.dhcp_options_id().map(|s| s.to_string()),
        }
    }

    pub fn state_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self.state.to_lowercase().as_str() {
            "available" => Color::Green,
            "pending" => Color::Yellow,
            _ => Color::Gray,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subnet {
    pub subnet_id: String,
    pub vpc_id: Option<String>,
    pub cidr_block: Option<String>,
    pub availability_zone: Option<String>,
    pub available_ip_count: Option<i32>,
    pub name: Option<String>,
    pub is_default: bool,
    pub map_public_ip: bool,
}

impl Subnet {
    pub fn from_aws(subnet: &aws_sdk_ec2::types::Subnet) -> Self {
        let name = subnet.tags()
            .iter()
            .find(|t| t.key() == Some("Name"))
            .and_then(|t| t.value())
            .map(|s| s.to_string());

        Self {
            subnet_id: subnet.subnet_id().unwrap_or_default().to_string(),
            vpc_id: subnet.vpc_id().map(|s| s.to_string()),
            cidr_block: subnet.cidr_block().map(|s| s.to_string()),
            availability_zone: subnet.availability_zone().map(|s| s.to_string()),
            available_ip_count: subnet.available_ip_address_count(),
            name,
            is_default: subnet.default_for_az().unwrap_or(false),
            map_public_ip: subnet.map_public_ip_on_launch().unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroup {
    pub group_id: String,
    pub group_name: String,
    pub description: Option<String>,
    pub vpc_id: Option<String>,
    pub inbound_rules_count: usize,
    pub outbound_rules_count: usize,
}

impl SecurityGroup {
    pub fn from_aws(sg: &aws_sdk_ec2::types::SecurityGroup) -> Self {
        Self {
            group_id: sg.group_id().unwrap_or_default().to_string(),
            group_name: sg.group_name().unwrap_or_default().to_string(),
            description: sg.description().map(|s| s.to_string()),
            vpc_id: sg.vpc_id().map(|s| s.to_string()),
            inbound_rules_count: sg.ip_permissions().len(),
            outbound_rules_count: sg.ip_permissions_egress().len(),
        }
    }
}
