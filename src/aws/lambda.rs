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


}

impl crate::aws::traits::AwsService<LambdaFunction> for LambdaService {
    fn list<'a>(&'a self) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<LambdaFunction>>> + Send + 'a>> {
        Box::pin(self.list_functions())
    }
}
