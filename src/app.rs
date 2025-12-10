use tokio::sync::mpsc;
use ratatui::Frame;
use crate::event::{Event, AwsEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    EC2,
    S3,
    RDS,
    DynamoDB,
    Lambda,
    VPC,
    IAM,
    Backup,
    CloudTrail,
}

pub enum Message {
    // Navigation
    NavigateToService(Service),
    NavigateBack,

    // Selection
    SelectNext,
    SelectPrevious,
    SelectItem(usize),

    // Actions
    RefreshData,
    ConfirmAction,
    CancelAction,

    // Async results
    // DataLoaded(ServiceData),
    // ActionCompleted(ActionResult),
    Error(String),

    // UI
    ToggleDetailPanel,
    ShowHelp,
    Quit,
}

pub struct App {
    pub should_quit: bool,
    pub current_service: Service,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            current_service: Service::EC2,
        }
    }

    pub async fn update(&mut self, message: Message, _event_tx: mpsc::UnboundedSender<Event>) {
        match message {
            Message::Quit => self.should_quit = true,
            Message::NavigateToService(service) => self.current_service = service,
            _ => {}
        }
    }

    pub fn handle_aws_event(&mut self, _event: AwsEvent) {
        // Handle AWS events
    }

    pub fn on_tick(&mut self) {
        // Handle tick
    }

    pub fn render(&self, frame: &mut Frame) {
        crate::ui::render::render(frame, self);
    }
}
