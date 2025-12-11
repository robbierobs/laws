//! Service screen modules and Screen trait implementations
//!
//! Each module provides a screen for viewing and interacting with AWS resources.

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

use ratatui::{layout::Rect, Frame};
use crate::app::{App, Service};
use crate::ui::Screen;

// Screen wrapper structs for each service
pub struct Ec2Screen;
pub struct S3Screen;
pub struct RdsScreen;
pub struct DynamoDbScreen;
pub struct LambdaScreen;
pub struct VpcScreen;
pub struct IamScreen;
pub struct BackupScreen;
pub struct CloudTrailScreen;
pub struct SecretsManagerScreen;

// Implement Screen trait for each service screen
impl Screen for Ec2Screen {
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
        ec2::render(frame, list_area, detail_area, app);
    }
}

impl Screen for S3Screen {
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
        s3::render(frame, list_area, detail_area, app);
    }
}

impl Screen for RdsScreen {
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
        rds::render(frame, list_area, detail_area, app);
    }
}

impl Screen for DynamoDbScreen {
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
        dynamodb::render(frame, list_area, detail_area, app);
    }
}

impl Screen for LambdaScreen {
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
        lambda::render(frame, list_area, detail_area, app);
    }
}

impl Screen for VpcScreen {
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
        vpc::render(frame, list_area, detail_area, app);
    }
}

impl Screen for IamScreen {
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
        iam::render(frame, list_area, detail_area, app);
    }
}

impl Screen for BackupScreen {
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
        backup::render(frame, list_area, detail_area, app);
    }
}

impl Screen for CloudTrailScreen {
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
        cloudtrail::render(frame, list_area, detail_area, app);
    }
}

impl Screen for SecretsManagerScreen {
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
        secretsmanager::render_secretsmanager_screen(frame, list_area, detail_area, app);
    }
}

/// Get the screen implementation for the given service
pub fn get_screen(service: Service) -> Box<dyn Screen> {
    match service {
        Service::EC2 => Box::new(Ec2Screen),
        Service::S3 => Box::new(S3Screen),
        Service::RDS => Box::new(RdsScreen),
        Service::DynamoDB => Box::new(DynamoDbScreen),
        Service::Lambda => Box::new(LambdaScreen),
        Service::VPC => Box::new(VpcScreen),
        Service::IAM => Box::new(IamScreen),
        Service::Backup => Box::new(BackupScreen),
        Service::CloudTrail => Box::new(CloudTrailScreen),
        Service::SecretsManager => Box::new(SecretsManagerScreen),
    }
}
