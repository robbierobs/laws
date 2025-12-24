//! Confirmable action trait
//!
//! Provides a standardized way for action enums to describe themselves
//! for confirmation modals.

/// Trait for actions that require user confirmation before execution
pub trait ConfirmableAction {
    /// Returns a human-readable description for the confirmation modal
    fn confirmation_description(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::messages::Ec2Action;

    #[test]
    fn test_ec2_start_description() {
        let action = Ec2Action::Start("i-12345".to_string());
        assert_eq!(action.confirmation_description(), "Start EC2 instance i-12345");
    }

    #[test]
    fn test_ec2_terminate_description() {
        let action = Ec2Action::Terminate("i-67890".to_string());
        assert_eq!(action.confirmation_description(), "Terminate EC2 instance i-67890");
    }
}
