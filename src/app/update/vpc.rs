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
    pub(super) fn handle_delete_security_group(&mut self, group_id: String, event_tx: crate::app::EventSender) {
        use crate::app::task_manager::task_keys;
        
        self.action_log.push(format!("Deleting Security Group: {}", group_id));
        
        self.spawn_aws_task(event_tx, task_keys::VPC_ACTION, move |clients, tx| async move {
            let service = crate::aws::vpc::VpcService::new(clients.ec2.clone());
            match service.delete_security_group(&group_id).await {
                Ok(_) => {
                    tx.send(crate::event::Event::Aws(Box::new(crate::event::AwsEvent::ActionCompleted(
                        format!("Security Group {} deleted", group_id)
                    )))).await.ok();
                    // Trigger refresh
                    tx.send(crate::event::Event::Message(crate::app::Message::refresh())).await.ok();
                }
                Err(e) => {
                    tx.send(crate::event::Event::Aws(Box::new(crate::event::AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });
    }
}
