//! S3 Bucket Creation modal input handling

use crate::app::states::s3::S3State;

use crossterm::event::{KeyCode, KeyEvent};

/// Result of handling S3 bucket creation input
pub enum S3BucketCreationResult {
    /// Stay in modal, no message
    Continue,
    /// Exit modal, no message
    Cancel,
    /// Exit modal with create bucket message
    Create(String),
}

/// Handle keyboard input for the S3 bucket creation modal.
pub fn handle_s3_bucket_creation_input(s3_state: &mut S3State, key: KeyEvent) -> S3BucketCreationResult {
    match key.code {
        KeyCode::Esc => {
            s3_state.reset_create_bucket_modal();
            S3BucketCreationResult::Cancel
        }
        KeyCode::Enter => {
            // Sanitize: lowercase, replace whitespace with hyphen, keep only valid chars
            let name: String = s3_state
                .create_bucket_input
                .trim()
                .to_lowercase()
                .chars()
                .map(|c| if c.is_whitespace() { '-' } else { c })
                .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-' || *c == '.')
                .collect();
            
            if !name.is_empty() {
                s3_state.show_create_bucket_modal = false;
                S3BucketCreationResult::Create(name)
            } else {
                S3BucketCreationResult::Continue
            }
        }
        KeyCode::Backspace => {
            s3_state.create_bucket_input.pop();
            S3BucketCreationResult::Continue
        }
        KeyCode::Char(c) => {
            // Allow any character, sanitize on submit
            s3_state.create_bucket_input.push(c);
            S3BucketCreationResult::Continue
        }
        _ => S3BucketCreationResult::Continue,
    }
}



