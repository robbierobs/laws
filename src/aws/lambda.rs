use crate::models::lambda::LambdaFunction;
use aws_sdk_lambda::Client;

pub struct LambdaService {
    client: Client,
}

fn format_lambda_error<E: std::fmt::Debug>(e: aws_sdk_lambda::error::SdkError<E>, action: &str, function_name: &str) -> anyhow::Error {
    let msg = if let Some(svc_err) = e.as_service_error() {
        let code = format!("{:?}", svc_err).split('{').next().unwrap_or("Unknown").trim().to_string();
        format!("Lambda {} failed for '{}': {}", action, function_name, code)
    } else {
        let err_str = format!("{}", e);
        if err_str.contains("Unhandled") || err_str.contains("unhandled") {
            format!("Lambda {} for '{}': Not supported (LocalStack limitation?)", action, function_name)
        } else {
            format!("Lambda {} failed for '{}': {}", action, function_name, err_str)
        }
    };
    anyhow::anyhow!(msg)
}

impl LambdaService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_functions(&self) -> anyhow::Result<Vec<LambdaFunction>> {
        let mut functions = Vec::new();
        let mut marker: Option<String> = None;

        loop {
            let mut request = self.client.list_functions();
            if let Some(m) = marker {
                request = request.marker(m);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format_lambda_error(e, "list_functions", "all"))?;

            for func in response.functions() {
                functions.push(LambdaFunction::from_aws(func));
            }

            marker = response.next_marker().map(|s| s.to_string());
            if marker.is_none() {
                break;
            }
        }

        Ok(functions)
    }

    pub async fn invoke_function(&self, function_name: &str) -> anyhow::Result<String> {
        let response = self.client
            .invoke()
            .function_name(function_name)
            .send()
            .await
            .map_err(|e| format_lambda_error(e, "invoke", function_name))?;

        let status = response.status_code();
        let payload = response.payload()
            .map(|p| String::from_utf8_lossy(p.as_ref()).to_string())
            .unwrap_or_else(|| "No payload".to_string());

        Ok(format!("Status: {}, Payload: {}", status, payload))
    }

    pub async fn get_function_url(&self, function_name: &str) -> anyhow::Result<Option<String>> {
        match self.client
            .get_function_url_config()
            .function_name(function_name)
            .send()
            .await
        {
            Ok(response) => Ok(Some(response.function_url().to_string())),
            Err(_) => Ok(None), // Function URL might not be configured
        }
    }
}
