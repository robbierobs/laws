//! Service screen modules and Screen trait implementations
//!
//! Each module provides a screen for viewing and interacting with AWS resources.

pub mod backup;
pub mod budgets;
pub mod cloudtrail;
pub mod dynamodb;
pub mod ec2;
pub mod ecr;
pub mod ecs;
pub mod iam;
pub mod lambda;
pub mod rds;
pub mod s3;
pub mod secretsmanager;
pub mod sqs;
pub mod vpc;

use crate::app::{App, Service};
use crate::ui::Screen;
use ratatui::{layout::Rect, Frame};

// Screen wrapper structs for each service
pub struct Ec2Screen;
pub struct S3Screen;
pub struct RdsScreen;
pub struct DynamoDbScreen;
pub struct LambdaScreen;
pub struct VpcScreen;
pub struct IamScreen;
pub struct BackupScreen;
pub struct BudgetsScreen;
pub struct CloudTrailScreen;
pub struct SecretsManagerScreen;
pub struct EcsScreen;
pub struct EcrScreen;
pub struct SqsScreen;

// Implement Screen trait for each service screen
impl Screen for Ec2Screen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        ec2::render(frame, list_area, detail_area, app);
    }
}

impl Screen for S3Screen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        s3::render(frame, list_area, detail_area, app);
    }
}

impl Screen for RdsScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        rds::render(frame, list_area, detail_area, app);
    }
}

impl Screen for DynamoDbScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        dynamodb::render(frame, list_area, detail_area, app);
    }
}

impl Screen for LambdaScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        lambda::render(frame, list_area, detail_area, app);
    }
}

impl Screen for VpcScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        vpc::render(frame, list_area, detail_area, app);
    }
}

impl Screen for IamScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        iam::render(frame, list_area, detail_area, app);
    }
}

impl Screen for BackupScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        backup::render(frame, list_area, detail_area, app);
    }
}

impl Screen for BudgetsScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        budgets::render(frame, list_area, detail_area, app);
    }
}

impl Screen for CloudTrailScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        cloudtrail::render(frame, list_area, detail_area, app);
    }
}

impl Screen for SecretsManagerScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        secretsmanager::render_secretsmanager_screen(frame, list_area, detail_area, app);
    }
}

impl Screen for EcsScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        // ECS screen handles its own splitting logic inside render usually or we simply pass area
        // But wait, list_area and detail_area are computed by ui::render.
        // My implementation in ecs.rs takes `render(frame, area, app)`.
        // It seems ui::render.rs splits the area.
        // Let's modify ecs.rs to assume full area or we adapt here.
        // The other screens seem to take list_area and detail_area.
        // Let's look at `ecs.rs` I wrote. It takes `area: Rect`.
        // So I should pick list_area (which might be the full area if detail is hidden)?

        // Wait, most screens like `ec2::render` probably handle list/detail logic themselves or take both.
        // Checking `ec2::render`: `pub fn render(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App)`
        // My `ecs::render` signature is `pub fn render(frame: &mut Frame, area: Rect, app: &App)`. This is wrong.
        // I need to fix `src/ui/screens/ecs.rs` signature first.

        // For now, I'll assume I will fix `ecs.rs` signature to match others.
        // Or I can call my simplistic `render` with whatever area I have.
        // But the standard here is list_area + detail_area.

        ecs::render(frame, list_area, detail_area, app);
    }
}

impl Screen for EcrScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        ecr::render(frame, list_area, detail_area, app);
    }
}

impl Screen for SqsScreen {
    fn render(
        &self,
        frame: &mut Frame,
        list_area: Option<Rect>,
        detail_area: Option<Rect>,
        app: &mut App,
    ) {
        sqs::render(frame, list_area, detail_area, app);
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
        Service::Budgets => Box::new(BudgetsScreen),
        Service::CloudTrail => Box::new(CloudTrailScreen),
        Service::SecretsManager => Box::new(SecretsManagerScreen),
        Service::ECS => Box::new(EcsScreen),
        Service::ECR => Box::new(EcrScreen),
        Service::SQS => Box::new(SqsScreen),
    }
}
