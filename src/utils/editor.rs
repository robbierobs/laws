//! Editor integration utilities
//!
//! Provides functions to open files in the user's preferred $EDITOR

use crate::error::AppResult;
use std::env;
use std::path::Path;
use std::process::Command;

/// Open a file in the user's preferred editor (from $EDITOR environment variable)
/// Falls back to 'vi' if $EDITOR is not set
///
/// # Arguments
/// * `file_path` - Path to the file to edit
///
/// # Returns
/// * `Ok(())` if the editor exited successfully
/// * `Err` if the editor failed or was not found
pub fn open_in_editor<P: AsRef<Path>>(file_path: P) -> AppResult<()> {
    let file_path = file_path.as_ref();
    
    // Get editor from environment variable, default to vi
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    
    // Try to run the editor
    let status = Command::new(&editor)
        .arg(file_path)
        .status()?;
    
    if !status.success() {
        return Err(crate::error::AppError::EditorFailed {
            editor,
            status_code: status.code(),
        });
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_open_in_editor_with_cat() {
        // Create a temporary file
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("lazy_aws_editor_test.txt");
        
        // Write some content
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);
        
        // Set EDITOR to 'cat' which will just display the file and exit successfully
        env::set_var("EDITOR", "cat");
        
        let result = open_in_editor(&test_file);
        
        // Clean up
        fs::remove_file(&test_file).ok();
        env::remove_var("EDITOR");
        
        assert!(result.is_ok(), "Expected successful editor invocation");
    }
    
    #[test]
    fn test_open_in_editor_with_nonexistent_editor() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("lazy_aws_editor_test2.txt");
        
        // Write some content
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);
        
        // Set EDITOR to a command that doesn't exist
        env::set_var("EDITOR", "this_editor_does_not_exist_xyz_123");
        
        let result = open_in_editor(&test_file);
        
        // Clean up
        fs::remove_file(&test_file).ok();
        env::remove_var("EDITOR");
        
        assert!(result.is_err(), "Expected error when editor doesn't exist");
    }
}
