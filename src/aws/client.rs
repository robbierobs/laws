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
    pub async fn new(profile: Option<&str>, region: Option<&str>) -> anyhow::Result<Self> {
        let mut config_loader = aws_config::defaults(BehaviorVersion::latest());
        
        if let Some(r) = region {
            config_loader = config_loader.region(Region::new(r.to_string()));
        }

        if let Some(profile_name) = profile {
            config_loader = config_loader.profile_name(profile_name);
        }

        let config = config_loader.load().await;

        Ok(Self {
            ec2: Ec2Client::new(&config),
            s3: S3Client::new(&config),
            rds: RdsClient::new(&config),
            dynamodb: DynamoDbClient::new(&config),
            lambda: LambdaClient::new(&config),
            iam: IamClient::new(&config),
            backup: BackupClient::new(&config),
            cloudtrail: CloudTrailClient::new(&config),
        })
    }
}
