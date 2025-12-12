pub mod ec2;
pub mod s3;
pub mod rds;
pub mod dynamodb;
pub mod lambda;
pub mod vpc;
pub mod iam;
pub mod backup;
pub mod cloudtrail;
pub mod secretsmanager;
pub mod ecs;
pub mod ecr;

use self::ec2::Ec2State;
use self::s3::S3State;
use self::rds::RdsState;
use self::dynamodb::DynamoDbState;
use self::lambda::LambdaState;
use self::vpc::VpcState;
use self::iam::IamState;
use self::backup::BackupState;
use self::cloudtrail::CloudTrailState;
use self::secretsmanager::SecretsManagerState;
use self::ecs::EcsState;
use self::ecr::EcrState;

/// Container for all service-specific states
///
/// This consolidates what was previously 50+ individual fields in the App struct
/// into a single organized structure with clear ownership.
#[derive(Default)]
pub struct ServiceStates {
    pub ec2: Ec2State,
    pub s3: S3State,
    pub rds: RdsState,
    pub dynamodb: DynamoDbState,
    pub lambda: LambdaState,
    pub vpc: VpcState,
    pub iam: IamState,
    pub backup: BackupState,
    pub cloudtrail: CloudTrailState,
    pub secretsmanager: SecretsManagerState,
    pub ecs: EcsState,
    pub ecr: EcrState,
}

impl ServiceStates {
    pub fn new() -> Self {
        Self {
            ec2: Ec2State::new(),
            s3: S3State::new(),
            rds: RdsState::new(),
            dynamodb: DynamoDbState::new(),
            lambda: LambdaState::new(),
            vpc: VpcState::new(),
            iam: IamState::new(),
            backup: BackupState::new(),
            cloudtrail: CloudTrailState::new(),
            secretsmanager: SecretsManagerState::new(),
            ecs: EcsState::new(),
            ecr: EcrState::new(),
        }
    }
}
