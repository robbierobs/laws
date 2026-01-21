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
pub mod budgets;
pub mod traits;

pub use self::traits::ServiceInternal;


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
use self::budgets::BudgetsState;

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
    pub budgets: BudgetsState,
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
            budgets: BudgetsState::new(),
        }
    }
}

impl crate::app::global_search::Searchable for ServiceStates {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        let mut results = Vec::new();
        results.extend(self.ec2.get_search_results());
        results.extend(self.s3.get_search_results());
        results.extend(self.rds.get_search_results());
        results.extend(self.dynamodb.get_search_results());
        results.extend(self.lambda.get_search_results());
        results.extend(self.vpc.get_search_results());
        results.extend(self.iam.get_search_results());
        results.extend(self.backup.get_search_results());
        results.extend(self.cloudtrail.get_search_results());
        results.extend(self.secretsmanager.get_search_results());
        results.extend(self.ecs.get_search_results());
        results.extend(self.ecr.get_search_results());
        results.extend(self.budgets.get_search_results());
        results
    }
}

impl ServiceStates {
    /// Select a resource by service and ID
    /// Returns true if the resource was found and selected
    pub fn select_by_service_and_id(&mut self, service: crate::app::Service, resource_id: &str) -> bool {
        self.get_mut(service).select_by_id(resource_id)
    }

    /// Clear all service states
    pub fn clear_all(&mut self) {
        for service in self.get_all_mut() {
            service.clear();
        }
    }

    /// Get all service states as mutable trait objects
    pub fn get_all_mut(&mut self) -> Vec<&mut dyn ServiceInternal> {
        vec![
            &mut self.ec2,
            &mut self.s3,
            &mut self.rds,
            &mut self.dynamodb,
            &mut self.lambda,
            &mut self.vpc,
            &mut self.iam,
            &mut self.backup,
            &mut self.cloudtrail,
            &mut self.secretsmanager,
            &mut self.ecs,
            &mut self.ecr,
            &mut self.budgets,
        ]
    }

    /// Get a specific service state by Service enum (mutable)
    ///
    /// This enables generic operations on services without repeating match statements.
    pub fn get_mut(&mut self, service: crate::app::Service) -> &mut dyn ServiceInternal {
        use crate::app::Service;
        match service {
            Service::EC2 => &mut self.ec2,
            Service::S3 => &mut self.s3,
            Service::RDS => &mut self.rds,
            Service::DynamoDB => &mut self.dynamodb,
            Service::Lambda => &mut self.lambda,
            Service::VPC => &mut self.vpc,
            Service::IAM => &mut self.iam,
            Service::Backup => &mut self.backup,
            Service::CloudTrail => &mut self.cloudtrail,
            Service::SecretsManager => &mut self.secretsmanager,
            Service::ECS => &mut self.ecs,
            Service::ECR => &mut self.ecr,
            Service::Budgets => &mut self.budgets,
        }
    }

    /// Get a specific service state by Service enum (immutable)
    #[allow(dead_code)] // Infrastructure for future use
    pub fn get(&self, service: crate::app::Service) -> &dyn ServiceInternal {
        use crate::app::Service;
        match service {
            Service::EC2 => &self.ec2,
            Service::S3 => &self.s3,
            Service::RDS => &self.rds,
            Service::DynamoDB => &self.dynamodb,
            Service::Lambda => &self.lambda,
            Service::VPC => &self.vpc,
            Service::IAM => &self.iam,
            Service::Backup => &self.backup,
            Service::CloudTrail => &self.cloudtrail,
            Service::SecretsManager => &self.secretsmanager,
            Service::ECS => &self.ecs,
            Service::ECR => &self.ecr,
            Service::Budgets => &self.budgets,
        }
    }
}
