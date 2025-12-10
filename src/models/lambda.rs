use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaFunction {
    pub function_name: String,
    pub function_arn: Option<String>,
    pub runtime: Option<String>,
    pub handler: Option<String>,
    pub code_size: i64,
    pub description: Option<String>,
    pub timeout: Option<i32>,
    pub memory_size: Option<i32>,
    pub last_modified: Option<String>,
    pub role: Option<String>,
    pub version: Option<String>,
    pub package_type: Option<String>,
    pub architectures: Vec<String>,
    pub state: Option<String>,
    pub state_reason: Option<String>,
    // VPC Config
    pub vpc_id: Option<String>,
    pub subnet_ids: Vec<String>,
    pub security_group_ids: Vec<String>,
    // Environment
    pub environment_variables: Vec<(String, String)>,
    // Layers
    pub layers: Vec<String>,
    // Ephemeral storage
    pub ephemeral_storage_size: Option<i32>,
}

impl LambdaFunction {
    pub fn from_aws(func: &aws_sdk_lambda::types::FunctionConfiguration) -> Self {
        let architectures: Vec<String> = func.architectures()
            .iter()
            .map(|a| a.as_str().to_string())
            .collect();

        let (vpc_id, subnet_ids, security_group_ids) = func.vpc_config()
            .map(|vpc| (
                vpc.vpc_id().map(|s| s.to_string()),
                vpc.subnet_ids().iter().map(|s| s.to_string()).collect(),
                vpc.security_group_ids().iter().map(|s| s.to_string()).collect(),
            ))
            .unwrap_or((None, Vec::new(), Vec::new()));

        let environment_variables: Vec<(String, String)> = func.environment()
            .and_then(|env| env.variables())
            .map(|vars| vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let layers: Vec<String> = func.layers()
            .iter()
            .filter_map(|l| l.arn().map(|s| s.to_string()))
            .collect();

        Self {
            function_name: func.function_name().unwrap_or_default().to_string(),
            function_arn: func.function_arn().map(|s| s.to_string()),
            runtime: func.runtime().map(|r| r.as_str().to_string()),
            handler: func.handler().map(|s| s.to_string()),
            code_size: func.code_size(),
            description: func.description().map(|s| s.to_string()),
            timeout: func.timeout(),
            memory_size: func.memory_size(),
            last_modified: func.last_modified().map(|s| s.to_string()),
            role: func.role().map(|s| s.to_string()),
            version: func.version().map(|s| s.to_string()),
            package_type: func.package_type().map(|p| p.as_str().to_string()),
            architectures,
            state: func.state().map(|s| s.as_str().to_string()),
            state_reason: func.state_reason().map(|s| s.to_string()),
            vpc_id,
            subnet_ids,
            security_group_ids,
            environment_variables,
            layers,
            ephemeral_storage_size: func.ephemeral_storage().map(|e| e.size()),
        }
    }
    
    pub fn state_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self.state.as_deref().unwrap_or("Unknown").to_uppercase().as_str() {
            "ACTIVE" => Color::Green,
            "PENDING" => Color::Yellow,
            "INACTIVE" => Color::Gray,
            "FAILED" => Color::Red,
            _ => Color::Gray,
        }
    }

    pub fn format_code_size(&self) -> String {
        const KB: i64 = 1024;
        const MB: i64 = KB * 1024;

        if self.code_size >= MB {
            format!("{:.2} MB", self.code_size as f64 / MB as f64)
        } else if self.code_size >= KB {
            format!("{:.2} KB", self.code_size as f64 / KB as f64)
        } else {
            format!("{} B", self.code_size)
        }
    }
}
