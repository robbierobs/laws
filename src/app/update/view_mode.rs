//! View mode cycling handlers
//!
//! Handles cycling through view modes for multi-view services.

use super::super::{App, Service, ViewMode};

impl App {
    pub(super) fn handle_cycle_view_mode(&mut self, forward: bool) {
        match self.current_service {
            Service::Backup => {
                self.services.backup.view_mode = if forward {
                    self.services.backup.view_mode.next()
                } else {
                    self.services.backup.view_mode.prev()
                };
                self.services.backup.list_state.select(None);
            }
            Service::CloudTrail => {
                self.services.cloudtrail.view_mode = if forward {
                    self.services.cloudtrail.view_mode.next()
                } else {
                    self.services.cloudtrail.view_mode.prev()
                };
                self.services.cloudtrail.list_state.select(None);
            }
            Service::VPC => {
                self.services.vpc.view_mode = if forward {
                    self.services.vpc.view_mode.next()
                } else {
                    self.services.vpc.view_mode.prev()
                };
                self.services.vpc.list_state.select(None);
            }
            Service::IAM => {
                self.services.iam.view_mode = if forward {
                    self.services.iam.view_mode.next()
                } else {
                    self.services.iam.view_mode.prev()
                };
                self.services.iam.list_state.select(None);
            }
            _ => {}
        }
    }
}
