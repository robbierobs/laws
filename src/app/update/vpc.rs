//! VPC update handlers
//!
//! Handles VPC-specific state mutations and async operations.

use super::super::{App, VpcViewMode};

impl App {
    pub(super) fn handle_drill_down_security_group(&mut self) {
        let Some(idx) = self.services.vpc.list_state.selected() else {
            return;
        };

        let Some(sg) = self.services.vpc.security_groups.get(idx) else {
            return;
        };

        self.services.vpc.selected_sg_id = Some(sg.group_id.clone());
        self.services.vpc.sg_rules_inbound = true;
        self.services.vpc.current_sg_rules = sg.inbound_rules.clone();
        self.services.vpc.view_mode = VpcViewMode::SecurityGroupRules;
        self.services.vpc.list_state.select(Some(0));
    }

    pub(super) fn handle_vpc_exit_sg_rules(&mut self) {
        self.services.vpc.view_mode = VpcViewMode::SecurityGroups;
        self.services.vpc.selected_sg_id = None;
        self.services.vpc.current_sg_rules.clear();
        self.services.vpc.list_state.select(Some(0));
    }

    pub(super) fn handle_toggle_sg_rules_direction(&mut self) {
        let Some(sg_id) = &self.services.vpc.selected_sg_id.clone() else {
            return;
        };

        let Some(sg) = self
            .services
            .vpc
            .security_groups
            .iter()
            .find(|s| &s.group_id == sg_id)
        else {
            return;
        };

        self.services.vpc.sg_rules_inbound = !self.services.vpc.sg_rules_inbound;
        self.services.vpc.current_sg_rules = if self.services.vpc.sg_rules_inbound {
            sg.inbound_rules.clone()
        } else {
            sg.outbound_rules.clone()
        };
        self.services.vpc.list_state.select(Some(0));
    }
}
