use ratatui::widgets::TableState;
use crate::models::vpc::{SecurityGroup, SecurityGroupRule, Subnet, Vpc};
use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt, VpcViewMode, EventSender};
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::aws::vpc::VpcService;
use crate::event::{AwsEvent, Event};
use crossterm::event::{KeyCode, KeyEvent};

/// State for VPC service
#[derive(Default)]
pub struct VpcState {
    pub vpcs: Vec<Vpc>,
    pub subnets: Vec<Subnet>,
    pub security_groups: Vec<SecurityGroup>,
    pub list_state: TableState,
    pub view_mode: VpcViewMode,
    pub current_sg_rules: Vec<SecurityGroupRule>,
    pub selected_sg_id: Option<String>,
    pub sg_rules_inbound: bool,
}

impl VpcState {
    pub fn new() -> Self {
        Self {
            sg_rules_inbound: true, // Default to showing inbound rules
            ..Self::default()
        }
    }

    /// Get the currently selected VPC, if any
    pub fn selected_vpc(&self) -> Option<&Vpc> {
        if self.view_mode == VpcViewMode::Vpcs {
            self.list_state.selected().and_then(|i| self.vpcs.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected subnet, if any
    pub fn selected_subnet(&self) -> Option<&Subnet> {
        if self.view_mode == VpcViewMode::Subnets {
            self.list_state.selected().and_then(|i| self.subnets.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected security group, if any
    pub fn selected_security_group(&self) -> Option<&SecurityGroup> {
        if self.view_mode == VpcViewMode::SecurityGroups {
            self.list_state
                .selected()
                .and_then(|i| self.security_groups.get(i))
        } else {
            None
        }
    }
}

impl crate::app::global_search::Searchable for VpcState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();

        // VPCs
        for vpc in &self.vpcs {
            let name = vpc.name.clone().unwrap_or_default();
            let mut result = SearchResult::new(Service::VPC, "VPC", &vpc.vpc_id);
            if !name.is_empty() {
                result = result.with_secondary(name);
            }
            results.push(result);
        }

        // Subnets
        for subnet in &self.subnets {
            let name = subnet.name.clone().unwrap_or_default();
            let mut result = SearchResult::new(Service::VPC, "Subnet", &subnet.subnet_id);
            if !name.is_empty() {
                result = result.with_secondary(name);
            }
            results.push(result);
        }

        // Security Groups
        for sg in &self.security_groups {
            let mut result = SearchResult::new(Service::VPC, "Security Group", &sg.group_id);
            result = result.with_secondary(sg.group_name.clone());
            results.push(result);
        }

        results
    }
}

impl crate::app::global_search::AutoSelectable for VpcState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        // Detect resource type from ID prefix
        if resource_id.starts_with("vpc-") {
            self.view_mode = VpcViewMode::Vpcs;
            if let Some(idx) = self.vpcs.iter().position(|v| v.vpc_id == resource_id) {
                self.list_state.select(Some(idx));
                return true;
            }
        } else if resource_id.starts_with("subnet-") {
            self.view_mode = VpcViewMode::Subnets;
            if let Some(idx) = self.subnets.iter().position(|s| s.subnet_id == resource_id) {
                self.list_state.select(Some(idx));
                return true;
            }
        } else if resource_id.starts_with("sg-") {
            self.view_mode = VpcViewMode::SecurityGroups;
            if let Some(idx) = self.security_groups.iter().position(|s| s.group_id == resource_id) {
                self.list_state.select(Some(idx));
                return true;
            }
        }
        false
    }
}

impl ServiceInputHandler for VpcState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        let len = match self.view_mode {
            VpcViewMode::Vpcs => self.vpcs.len(),
            VpcViewMode::Subnets => self.subnets.len(),
            VpcViewMode::SecurityGroups => self.security_groups.len(),
            VpcViewMode::SecurityGroupRules => self.current_sg_rules.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter if self.view_mode == VpcViewMode::SecurityGroups => {
                return InputResult::Message(Message::vpc_drill_down_sg())
            }
            KeyCode::Esc if self.view_mode == VpcViewMode::SecurityGroupRules => {
                return InputResult::Message(Message::vpc_exit_sg_rules())
            }
            // Toggle between inbound and outbound rules with 't' or 'v'
            KeyCode::Char('t') | KeyCode::Char('v')
                if self.view_mode == VpcViewMode::SecurityGroupRules =>
            {
                return InputResult::Message(Message::vpc_toggle_sg_rules_direction());
            }
            KeyCode::Char('X') | KeyCode::Delete
                if self.view_mode == VpcViewMode::SecurityGroups =>
            {
                if let Some(sg) = self.selected_security_group() {
                    return InputResult::Action(Message::vpc_delete_security_group(
                        sg.group_id.clone(),
                    ));
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }

    fn get_copiable_text(&self) -> Option<String> {
        match self.view_mode {
            VpcViewMode::Vpcs => self.selected_vpc().map(|v| v.vpc_id.clone()),
            VpcViewMode::Subnets => self.selected_subnet().map(|s| s.subnet_id.clone()),
            VpcViewMode::SecurityGroups => self.selected_security_group().map(|sg| sg.group_id.clone()),
            VpcViewMode::SecurityGroupRules => None,
        }
    }
}

impl ServiceInternal for VpcState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.ec2.clone();
        let handle = tokio::spawn(async move {
            let service = VpcService::new(client);
            if let Ok(vpcs) = service.list_vpcs().await {
                tx.send(Event::Aws(Box::new(AwsEvent::VpcsLoaded(vpcs))))
                    .await
                    .ok();
            } else if report_errors {
                tx.send(Event::Aws(Box::new(AwsEvent::Error(
                    "Failed to load VPCs".to_string(),
                ))))
                .await
                .ok();
            }
            if let Ok(subnets) = service.list_subnets(None).await {
                tx.send(Event::Aws(Box::new(AwsEvent::SubnetsLoaded(subnets))))
                    .await
                    .ok();
            }
            if let Ok(sgs) = service.list_security_groups(None).await {
                tx.send(Event::Aws(Box::new(AwsEvent::SecurityGroupsLoaded(sgs))))
                    .await
                    .ok();
            }
        });
        tasks.spawn(task_keys::VPC_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.vpcs.clear();
        self.subnets.clear();
        self.security_groups.clear();
        self.current_sg_rules.clear();
        self.selected_sg_id = None;
        self.view_mode = VpcViewMode::Vpcs;
        self.list_state.select(Some(0));
    }
}
