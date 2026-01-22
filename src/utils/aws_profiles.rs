//! AWS Profile and Region utilities
//!
//! Functions for reading AWS profiles from config files and providing
//! a complete list of AWS regions.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// All AWS regions, including GovCloud regions
pub const ALL_REGIONS: &[&str] = &[
    // US regions
    "us-east-1",
    "us-east-2",
    "us-west-1",
    "us-west-2",
    // GovCloud regions
    "us-gov-east-1",
    "us-gov-west-1",
    // Canada
    "ca-central-1",
    "ca-west-1",
    // Europe
    "eu-central-1",
    "eu-central-2",
    "eu-west-1",
    "eu-west-2",
    "eu-west-3",
    "eu-north-1",
    "eu-south-1",
    "eu-south-2",
    // Asia Pacific
    "ap-east-1",
    "ap-south-1",
    "ap-south-2",
    "ap-northeast-1",
    "ap-northeast-2",
    "ap-northeast-3",
    "ap-southeast-1",
    "ap-southeast-2",
    "ap-southeast-3",
    "ap-southeast-4",
    // Middle East
    "me-south-1",
    "me-central-1",
    // Africa
    "af-south-1",
    // South America
    "sa-east-1",
    // Israel
    "il-central-1",
];

/// Read AWS profiles from ~/.aws/config and ~/.aws/credentials
pub fn list_profiles() -> Vec<String> {
    let mut profiles = HashSet::new();

    // Always include "default" as an option
    profiles.insert("default".to_string());

    // Read from ~/.aws/config
    if let Some(config_path) = get_aws_config_path() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            for profile in parse_profiles_from_config(&content) {
                profiles.insert(profile);
            }
        }
    }

    // Read from ~/.aws/credentials
    if let Some(creds_path) = get_aws_credentials_path() {
        if let Ok(content) = fs::read_to_string(&creds_path) {
            for profile in parse_profiles_from_credentials(&content) {
                profiles.insert(profile);
            }
        }
    }

    let mut sorted: Vec<String> = profiles.into_iter().collect();
    sorted.sort();

    // Move "default" to the front if it exists
    if let Some(pos) = sorted.iter().position(|p| p == "default") {
        let default = sorted.remove(pos);
        sorted.insert(0, default);
    }

    sorted
}

/// Get the path to ~/.aws/config
fn get_aws_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".aws").join("config"))
}

/// Get the path to ~/.aws/credentials
fn get_aws_credentials_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".aws").join("credentials"))
}

/// Parse profile names from ~/.aws/config
/// Config file uses [profile name] format (except for default which is just [default])
fn parse_profiles_from_config(content: &str) -> Vec<String> {
    let mut profiles = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len() - 1].trim();
            if *section == "default" {
                profiles.push("default".to_string());
            } else if let Some(profile_name) = section.strip_prefix("profile ") {
                profiles.push(profile_name.trim().to_string());
            }
        }
    }

    profiles
}

/// Parse profile names from ~/.aws/credentials
/// Credentials file uses [name] format directly
fn parse_profiles_from_credentials(content: &str) -> Vec<String> {
    let mut profiles = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let profile_name = &line[1..line.len() - 1].trim();
            profiles.push(profile_name.to_string());
        }
    }

    profiles
}

/// Check if a profile uses SSO authentication
pub fn is_sso_profile(profile_name: &str) -> bool {
    if let Some(config_path) = get_aws_config_path() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            return check_profile_has_sso(&content, profile_name);
        }
    }
    false
}

/// Parse the config file and check if the given profile has SSO settings
fn check_profile_has_sso(content: &str, profile_name: &str) -> bool {
    let target_section = if profile_name == "default" {
        "[default]".to_string()
    } else {
        format!("[profile {}]", profile_name)
    };

    let mut in_target_section = false;

    for line in content.lines() {
        let line = line.trim();

        // Check for section header
        if line.starts_with('[') && line.ends_with(']') {
            in_target_section = line == target_section;
            continue;
        }

        // Check for SSO-related keys in the target section
        if in_target_section
            && (line.starts_with("sso_start_url")
                || line.starts_with("sso_account_id")
                || line.starts_with("sso_role_name")
                || line.starts_with("sso_session"))
        {
            return true;
        }
    }

    false
}

/// Get the endpoint_url configured for a profile in ~/.aws/config
/// This is used for LocalStack and other custom endpoints
pub fn get_profile_endpoint_url(profile_name: &str) -> Option<String> {
    if let Some(config_path) = get_aws_config_path() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            return parse_profile_endpoint_url(&content, profile_name);
        }
    }
    None
}

/// Parse the config file and extract endpoint_url for a given profile
fn parse_profile_endpoint_url(content: &str, profile_name: &str) -> Option<String> {
    let target_section = if profile_name == "default" {
        "[default]".to_string()
    } else {
        format!("[profile {}]", profile_name)
    };

    let mut in_target_section = false;

    for line in content.lines() {
        let line = line.trim();

        // Check for section header
        if line.starts_with('[') && line.ends_with(']') {
            in_target_section = line == target_section;
            continue;
        }

        // Check for endpoint_url in the target section
        if in_target_section {
            if let Some(value) = line.strip_prefix("endpoint_url") {
                // Handle both "endpoint_url = value" and "endpoint_url=value"
                let value = value.trim().trim_start_matches('=').trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_profiles_from_config() {
        let content = r#"
[default]
region = us-east-1

[profile dev]
region = us-west-2

[profile prod]
region = eu-west-1
"#;
        let profiles = parse_profiles_from_config(content);
        assert!(profiles.contains(&"default".to_string()));
        assert!(profiles.contains(&"dev".to_string()));
        assert!(profiles.contains(&"prod".to_string()));
    }

    #[test]
    fn test_parse_profiles_from_credentials() {
        let content = r#"
[default]
aws_access_key_id = AKIA...
aws_secret_access_key = secret

[dev]
aws_access_key_id = AKIA...
aws_secret_access_key = secret
"#;
        let profiles = parse_profiles_from_credentials(content);
        assert!(profiles.contains(&"default".to_string()));
        assert!(profiles.contains(&"dev".to_string()));
    }

    #[test]
    fn test_check_profile_has_sso() {
        let content = r#"
[default]
region = us-east-1

[profile sso-profile]
sso_start_url = https://my-sso-portal.awsapps.com/start
sso_region = us-east-1
sso_account_id = 123456789012
sso_role_name = MyRole
region = us-east-1
"#;
        assert!(!check_profile_has_sso(content, "default"));
        assert!(check_profile_has_sso(content, "sso-profile"));
    }

    #[test]
    fn test_parse_profile_endpoint_url() {
        let content = r#"
[default]
region = us-east-1

[profile localstack]
region = us-east-1
endpoint_url = http://localhost.localstack.cloud:4566

[profile other]
region = us-west-2
"#;
        assert!(parse_profile_endpoint_url(content, "default").is_none());
        assert_eq!(
            parse_profile_endpoint_url(content, "localstack"),
            Some("http://localhost.localstack.cloud:4566".to_string())
        );
        assert!(parse_profile_endpoint_url(content, "other").is_none());
    }

    #[test]
    fn test_parse_profile_endpoint_url_no_spaces() {
        let content = r#"
[profile localstack]
endpoint_url=http://localhost:4566
"#;
        assert_eq!(
            parse_profile_endpoint_url(content, "localstack"),
            Some("http://localhost:4566".to_string())
        );
    }

    #[test]
    fn test_all_regions_includes_govcloud() {
        assert!(ALL_REGIONS.contains(&"us-gov-east-1"));
        assert!(ALL_REGIONS.contains(&"us-gov-west-1"));
    }
}
