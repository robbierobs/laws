use crate::error::AppResult;
use crate::models::lambda::LambdaFunction;
use crate::utils::error::format_sdk_error;
use aws_sdk_lambda::Client;

pub struct LambdaService {
    client: Client,
}

impl LambdaService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_functions(&self) -> AppResult<Vec<LambdaFunction>> {
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
                .map_err(|e| format_sdk_error("Lambda", "list_functions", "all", e))?;

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

    pub async fn invoke_function(&self, function_name: &str) -> AppResult<String> {
        let response = self.client
            .invoke()
            .function_name(function_name)
            .send()
            .await
            .map_err(|e| format_sdk_error("Lambda", "invoke", function_name, e))?;

        let payload = response.payload()
            .map(|b| String::from_utf8_lossy(b.as_ref()).to_string())
            .unwrap_or_else(|| "No payload".to_string());
            
        Ok(payload)
    }

    pub async fn delete_function(&self, function_name: &str) -> AppResult<()> {
        self.client
            .delete_function()
            .function_name(function_name)
            .send()
            .await
            .map_err(|e| format_sdk_error("Lambda", "delete_function", function_name, e))?;
        Ok(())
    }

    pub async fn get_function_details(&self, function_name: &str) -> AppResult<crate::models::lambda::LambdaFunctionDetails> {
        let response = self.client
            .get_function()
            .function_name(function_name)
            .send()
            .await
            .map_err(|e| format_sdk_error("Lambda", "get_function", function_name, e))?;

        let mut tags: Vec<(String, String)> = response.tags()
            .into_iter()
            .flat_map(|m| m.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        tags.sort_by(|a, b| a.0.cmp(&b.0));

        let concurrency = response.concurrency().and_then(|c| c.reserved_concurrent_executions());

        Ok(crate::models::lambda::LambdaFunctionDetails {
            tags,
            concurrency,
            last_update_status: response.configuration().and_then(|c| c.last_update_status()).map(|s| s.as_str().to_string()),
            last_update_status_reason: response.configuration().and_then(|c| c.last_update_status_reason()).map(|s| s.to_string()),
            loading: false,
        })
    }

}

impl crate::aws::traits::AwsService<LambdaFunction> for LambdaService {
    fn list<'a>(&'a self) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<LambdaFunction>>> + Send + 'a>> {
        Box::pin(self.list_functions())
    }
}
