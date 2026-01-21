use aws_sdk_ec2::types::IpPermission;
use serde::{Deserialize, Serialize};

use crate::models::find_name_tag;

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
        let name = find_name_tag(vpc.tags().iter());

        Self {
            vpc_id: vpc.vpc_id().unwrap_or_default().to_string(),
            cidr_block: vpc.cidr_block().map(|s| s.to_string()),
            state: vpc
                .state()
                .map(|s| s.as_str().to_string())
                .unwrap_or_default(),
            is_default: vpc.is_default().unwrap_or(false),
            name,
            owner_id: vpc.owner_id().map(|s| s.to_string()),
            instance_tenancy: vpc.instance_tenancy().map(|t| t.as_str().to_string()),
            dhcp_options_id: vpc.dhcp_options_id().map(|s| s.to_string()),
        }
    }

    pub fn state_color(&self) -> ratatui::style::Color {
        use crate::ui::theme::THEME;
        match self.state.to_lowercase().as_str() {
            "available" => THEME.success,
            "pending" => THEME.warning,
            _ => THEME.muted,
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
        let name = find_name_tag(subnet.tags().iter());

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
    pub inbound_rules: Vec<SecurityGroupRule>,
    pub outbound_rules: Vec<SecurityGroupRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroupRule {
    pub protocol: String,
    pub port_range: String,
    pub source: String,
    pub description: Option<String>,
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
            inbound_rules: sg
                .ip_permissions()
                .iter()
                .flat_map(SecurityGroupRule::from_aws)
                .collect(),
            outbound_rules: sg
                .ip_permissions_egress()
                .iter()
                .flat_map(SecurityGroupRule::from_aws)
                .collect(),
        }
    }
}

impl SecurityGroupRule {
    pub fn from_aws(perm: &IpPermission) -> Vec<Self> {
        let mut rules = Vec::new();
        let protocol = perm.ip_protocol().unwrap_or("-").to_string();
        let port_range = if let (Some(from), Some(to)) = (perm.from_port(), perm.to_port()) {
            if from == to {
                from.to_string()
            } else {
                format!("{}-{}", from, to)
            }
        } else {
            "All".to_string()
        };

        // IP Ranges
        for range in perm.ip_ranges() {
            rules.push(Self {
                protocol: protocol.clone(),
                port_range: port_range.clone(),
                source: range.cidr_ip().unwrap_or("-").to_string(),
                description: range.description().map(|s| s.to_string()),
            });
        }

        // IPv6 Ranges
        for range in perm.ipv6_ranges() {
            rules.push(Self {
                protocol: protocol.clone(),
                port_range: port_range.clone(),
                source: range.cidr_ipv6().unwrap_or("-").to_string(),
                description: range.description().map(|s| s.to_string()),
            });
        }

        // User Id Group Pairs (Source SGs)
        for pair in perm.user_id_group_pairs() {
            rules.push(Self {
                protocol: protocol.clone(),
                port_range: port_range.clone(),
                source: pair.group_id().unwrap_or("-").to_string(),
                description: pair.description().map(|s| s.to_string()),
            });
        }

        // Prefix List Ids
        for prefix in perm.prefix_list_ids() {
            rules.push(Self {
                protocol: protocol.clone(),
                port_range: port_range.clone(),
                source: prefix.prefix_list_id().unwrap_or("-").to_string(),
                description: prefix.description().map(|s| s.to_string()),
            });
        }

        // If no specific source, but protocol is present (e.g. all traffic allowed implicitly or explicitly without ranges?)
        // Usually there is at least one range or group. If empty, it might mean no rules?
        // But IpPermission usually groups by protocol/port.

        rules
    }
}

impl crate::models::Filterable for Vpc {
    fn matches_filter(&self, filter: &str) -> bool {
        self.vpc_id.to_lowercase().contains(filter)
            || self
                .name
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
            || self
                .cidr_block
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
    }
}

impl crate::models::Filterable for Subnet {
    fn matches_filter(&self, filter: &str) -> bool {
        self.subnet_id.to_lowercase().contains(filter)
            || self
                .name
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
            || self
                .cidr_block
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
    }
}

impl crate::models::Filterable for SecurityGroup {
    fn matches_filter(&self, filter: &str) -> bool {
        self.group_id.to_lowercase().contains(filter)
            || self.group_name.to_lowercase().contains(filter)
            || self
                .description
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
    }
}

impl crate::models::Filterable for SecurityGroupRule {
    fn matches_filter(&self, filter: &str) -> bool {
        self.protocol.to_lowercase().contains(filter)
            || self.source.to_lowercase().contains(filter)
            || self
                .description
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
    }
}
