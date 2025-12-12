use ratatui::widgets::TableState;
use crate::models::iam::{IamPolicy, IamRole, IamUser};
use crate::app::{IamViewMode, InputResult, Message, ServiceInputHandler, TableStateExt};
use crossterm::event::{KeyCode, KeyEvent};

/// State for IAM service
#[derive(Default)]
pub struct IamState {
    pub roles: Vec<IamRole>,
    pub users: Vec<IamUser>,
    pub policies: Vec<IamPolicy>,
    pub list_state: TableState,
    pub view_mode: IamViewMode,
    pub previous_view_mode: IamViewMode,
    pub current_policies: Vec<IamPolicy>,
    pub current_policy_document: String,
    pub selected_entity_name: Option<String>,
}

impl IamState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected user, if any
    pub fn selected_user(&self) -> Option<&IamUser> {
        if self.view_mode == IamViewMode::Users {
            self.list_state.selected().and_then(|i| self.users.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected role, if any
    pub fn selected_role(&self) -> Option<&IamRole> {
        if self.view_mode == IamViewMode::Roles {
            self.list_state.selected().and_then(|i| self.roles.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected policy, if any
    pub fn selected_policy(&self) -> Option<&IamPolicy> {
        if self.view_mode == IamViewMode::Policies {
            self.list_state
                .selected()
                .and_then(|i| self.policies.get(i))
        } else {
            None
        }
    }
}

impl ServiceInputHandler for IamState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        let len = match self.view_mode {
            IamViewMode::Users => self.users.len(),
            IamViewMode::Roles => self.roles.len(),
            IamViewMode::Policies => self.policies.len(),
            IamViewMode::UserAttachedPolicies | IamViewMode::RoleAttachedPolicies => {
                self.current_policies.len()
            }
            IamViewMode::PolicyDocument => 0,
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter => {
                return match self.view_mode {
                    IamViewMode::Users => InputResult::Message(Message::iam_drill_down_user()),
                    IamViewMode::Roles => InputResult::Message(Message::iam_drill_down_role()),
                    IamViewMode::Policies
                    | IamViewMode::UserAttachedPolicies
                    | IamViewMode::RoleAttachedPolicies => {
                        InputResult::Message(Message::iam_drill_down_policy())
                    }
                    IamViewMode::PolicyDocument => InputResult::None,
                };
            }
            KeyCode::Esc if !self.view_mode.is_main_tab() => {
                return InputResult::Message(Message::iam_exit_drill_down())
            }
            KeyCode::Char('X') | KeyCode::Delete if self.view_mode.is_main_tab() => {
                match self.view_mode {
                    IamViewMode::Users => {
                        if let Some(user) = self.selected_user() {
                            return InputResult::Action(Message::iam_delete_user(
                                user.user_name.clone(),
                            ));
                        }
                    }
                    IamViewMode::Roles => {
                        if let Some(role) = self.selected_role() {
                            return InputResult::Action(Message::iam_delete_role(
                                role.role_name.clone(),
                            ));
                        }
                    }
                    IamViewMode::Policies => {
                        if let Some(policy) = self.selected_policy() {
                            if let Some(arn) = &policy.arn {
                                return InputResult::Action(Message::iam_delete_policy(
                                    arn.clone(),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        InputResult::None
    }
}
