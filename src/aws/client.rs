use aws_config::{BehaviorVersion, Region};
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_rds::Client as RdsClient;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_lambda::Client as LambdaClient;
use aws_sdk_iam::Client as IamClient;
use aws_sdk_backup::Client as BackupClient;
use aws_sdk_cloudtrail::Client as CloudTrailClient;

#[derive(Clone)]
pub struct AwsClients {
    pub ec2: Ec2Client,
    pub s3: S3Client,
    pub rds: RdsClient,
    pub dynamodb: DynamoDbClient,
    pub lambda: LambdaClient,
    pub iam: IamClient,
    pub backup: BackupClient,
    pub cloudtrail: CloudTrailClient,
}

impl AwsClients {
    pub async fn new(
        profile: Option<&str>, 
        region: Option<&str>,
        endpoint_url: Option<&str>,
    ) -> anyhow::Result<Self> {
        let mut config_loader = aws_config::defaults(BehaviorVersion::latest());
        
        // Set region: CLI arg > AWS_REGION env > default to us-east-1
        let region_str = region
            .map(|s| s.to_string())
            .or_else(|| std::env::var("AWS_REGION").ok())
            .or_else(|| std::env::var("AWS_DEFAULT_REGION").ok())
            .unwrap_or_else(|| "us-east-1".to_string());
        config_loader = config_loader.region(Region::new(region_str));

        // Set profile: CLI arg > AWS_PROFILE env
        let profile_name = profile
            .map(|s| s.to_string())
            .or_else(|| std::env::var("AWS_PROFILE").ok());
        
        if let Some(ref p) = profile_name {
            config_loader = config_loader.profile_name(p);
        }

        // Set custom endpoint URL: CLI arg > profile config > AWS_ENDPOINT_URL env
        // First check the profile's config file for endpoint_url
        let profile_endpoint = profile_name
            .as_ref()
            .and_then(|p| crate::utils::aws_profiles::get_profile_endpoint_url(p));
        
        let endpoint = endpoint_url
            .map(|s| s.to_string())
            .or(profile_endpoint)
            .or_else(|| std::env::var("AWS_ENDPOINT_URL").ok());
        
        if let Some(ref url) = endpoint {
            config_loader = config_loader.endpoint_url(url);
        }

        let config = config_loader.load().await;

        // For LocalStack/custom endpoints, we need to use path-style addressing for S3
        let s3_config = if endpoint.is_some() {
            aws_sdk_s3::config::Builder::from(&config)
                .force_path_style(true)
                .build()
        } else {
            aws_sdk_s3::config::Builder::from(&config).build()
        };

        Ok(Self {
            ec2: Ec2Client::new(&config),
            s3: S3Client::from_conf(s3_config),
            rds: RdsClient::new(&config),
            dynamodb: DynamoDbClient::new(&config),
            lambda: LambdaClient::new(&config),
            iam: IamClient::new(&config),
            backup: BackupClient::new(&config),
            cloudtrail: CloudTrailClient::new(&config),
        })
    }
}
