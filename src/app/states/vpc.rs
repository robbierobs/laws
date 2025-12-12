use ratatui::widgets::TableState;
use crate::models::vpc::{SecurityGroup, SecurityGroupRule, Subnet, Vpc};
use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt, VpcViewMode};
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
}
