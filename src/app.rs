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

impl Service {
    pub fn as_str(&self) -> &str {
        match self {
            Service::EC2 => "EC2",
            Service::S3 => "S3",
            Service::RDS => "RDS",
            Service::DynamoDB => "DynamoDB",
            Service::Lambda => "Lambda",
            Service::VPC => "VPC",
            Service::IAM => "IAM",
            Service::Backup => "Backup",
            Service::CloudTrail => "CloudTrail",
        }
    }

    pub fn iterator() -> impl Iterator<Item = Self> {
        [
            Self::EC2,
            Self::S3,
            Self::RDS,
            Self::DynamoDB,
            Self::Lambda,
            Self::VPC,
            Self::IAM,
            Self::Backup,
            Self::CloudTrail,
        ]
        .iter()
        .copied()
    }
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
    StartInstance(String),
    StopInstance(String),
    RebootInstance(String),
    TerminateInstance(String),

    // Async results
    // DataLoaded(ServiceData),
    // ActionCompleted(ActionResult),
    Error(String),

    // UI
    ToggleDetailPanel,
    ShowHelp,
    Quit,
}

use crate::ui::components::sidebar::Sidebar;
use crate::ui::components::Component;
use crossterm::event::{KeyCode, KeyEvent};

// ... (Service enum remains)

pub enum Focus {
    Sidebar,
    Main,
}

use crate::aws::client::AwsClients;
use crate::models::ec2::Ec2Instance;

use ratatui::widgets::TableState;

pub struct App {
    pub should_quit: bool,
    pub current_service: Service,
    pub sidebar: Sidebar,
    pub focus: Focus,
    pub aws_clients: Option<AwsClients>,
    pub ec2_instances: Vec<Ec2Instance>,
    pub ec2_list_state: TableState,
    pub loading: bool,
    pub should_refresh: bool,
}

impl App {
    pub fn new(aws_clients: Option<AwsClients>) -> Self {
        Self {
            should_quit: false,
            current_service: Service::EC2,
            sidebar: Sidebar::new(),
            focus: Focus::Sidebar,
            aws_clients,
            ec2_instances: Vec::new(),
            ec2_list_state: TableState::default(),
            loading: false,
            should_refresh: false,
        }
    }

    pub fn update<'a>(&'a mut self, message: Message, event_tx: mpsc::UnboundedSender<Event>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            match message {
                Message::Quit => self.should_quit = true,
                Message::NavigateToService(service) => {
                    self.current_service = service;
                    // Trigger data refresh when switching services
                    self.update(Message::RefreshData, event_tx).await;
                }
                Message::RefreshData => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        match self.current_service {
                            Service::EC2 => {
                                let client = clients.ec2.clone();
                                let tx = event_tx.clone();
                                tokio::spawn(async move {
                                    let service = crate::aws::ec2::Ec2Service::new(client);
                                    match service.list_instances().await {
                                        Ok(instances) => {
                                            tx.send(Event::Aws(AwsEvent::Ec2InstancesLoaded(instances))).ok();
                                        }
                                        Err(e) => {
                                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                        }
                                    }
                                });
                            }
                            _ => {
                                // TODO: Implement other services
                                self.loading = false;
                            }
                        }
                    }
                }
                Message::StartInstance(id) => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        let client = clients.ec2.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::ec2::Ec2Service::new(client);
                            match service.start_instance(&id).await {
                                Ok(_) => {
                                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Started instance {}", id)))).ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                }
                            }
                        });
                    }
                }
                Message::StopInstance(id) => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        let client = clients.ec2.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::ec2::Ec2Service::new(client);
                            match service.stop_instance(&id).await {
                                Ok(_) => {
                                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Stopped instance {}", id)))).ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                }
                            }
                        });
                    }
                }
                Message::RebootInstance(id) => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        let client = clients.ec2.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::ec2::Ec2Service::new(client);
                            match service.reboot_instance(&id).await {
                                Ok(_) => {
                                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Rebooted instance {}", id)))).ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                }
                            }
                        });
                    }
                }
                _ => {}
            }
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Message> {
        if key.code == KeyCode::Tab {
            self.toggle_focus();
            return None;
        }

        // Route to focused component
        match self.focus {
            Focus::Sidebar => {
                if let Some(msg) = self.sidebar.handle_key(key) {
                    return Some(msg);
                }
            }
            Focus::Main => {
                match self.current_service {
                    Service::EC2 => {
                        match key.code {
                            KeyCode::Down | KeyCode::Char('j') => {
                                let i = match self.ec2_list_state.selected() {
                                    Some(i) => {
                                        if i >= self.ec2_instances.len() - 1 {
                                            0
                                        } else {
                                            i + 1
                                        }
                                    }
                                    None => 0,
                                };
                                self.ec2_list_state.select(Some(i));
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                let i = match self.ec2_list_state.selected() {
                                    Some(i) => {
                                        if i == 0 {
                                            self.ec2_instances.len() - 1
                                        } else {
                                            i - 1
                                        }
                                    }
                                    None => 0,
                                };
                                self.ec2_list_state.select(Some(i));
                            }
                            KeyCode::Char('s') => {
                                if let Some(i) = self.ec2_list_state.selected() {
                                    if let Some(instance) = self.ec2_instances.get(i) {
                                        return Some(Message::StartInstance(instance.instance_id.clone()));
                                    }
                                }
                            }
                            KeyCode::Char('S') => {
                                if let Some(i) = self.ec2_list_state.selected() {
                                    if let Some(instance) = self.ec2_instances.get(i) {
                                        return Some(Message::StopInstance(instance.instance_id.clone()));
                                    }
                                }
                            }
                            KeyCode::Char('R') => {
                                if let Some(i) = self.ec2_list_state.selected() {
                                    if let Some(instance) = self.ec2_instances.get(i) {
                                        return Some(Message::RebootInstance(instance.instance_id.clone()));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Global keys
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Message::Quit),
            KeyCode::Char('1') => Some(Message::NavigateToService(Service::EC2)),
            KeyCode::Char('2') => Some(Message::NavigateToService(Service::S3)),
            KeyCode::Char('3') => Some(Message::NavigateToService(Service::RDS)),
            KeyCode::Char('4') => Some(Message::NavigateToService(Service::DynamoDB)),
            KeyCode::Char('5') => Some(Message::NavigateToService(Service::Lambda)),
            KeyCode::Char('6') => Some(Message::NavigateToService(Service::VPC)),
            KeyCode::Char('7') => Some(Message::NavigateToService(Service::IAM)),
            KeyCode::Char('8') => Some(Message::NavigateToService(Service::Backup)),
            KeyCode::Char('9') => Some(Message::NavigateToService(Service::CloudTrail)),
            _ => None,
        }
    }

    fn toggle_focus(&mut self) {
        match self.focus {
            Focus::Sidebar => {
                self.focus = Focus::Main;
                self.sidebar.is_focused = false;
            }
            Focus::Main => {
                self.focus = Focus::Sidebar;
                self.sidebar.is_focused = true;
            }
        }
    }

    pub fn handle_aws_event(&mut self, event: AwsEvent) {
        match event {
            AwsEvent::Ec2InstancesLoaded(instances) => {
                self.ec2_instances = instances;
                self.loading = false;
            }
            AwsEvent::ActionCompleted(msg) => {
                self.loading = false;
                // TODO: Show success message via a notification system
                eprintln!("Action completed: {}", msg);
                
                // Trigger refresh
                // Since we are in a synchronous method, we can't await. 
                // But we can spawn a task if we had a handle, or just set a flag.
                // For now, let's assume the next tick or user interaction will pick it up? 
                // No, we need to actively trigger it.
                // A common pattern is to have an `Action` queue or similar.
                // Or, we can just send a message to the event loop if we had the sender here.
                // But we don't have the sender in `handle_aws_event`.
                // Let's add a `should_refresh` flag to App and check it in `on_tick` or `update`.
                self.should_refresh = true;
            }
            AwsEvent::Error(e) => {
                // TODO: Show error in UI
                self.loading = false;
                eprintln!("Error: {}", e);
            }
        }
    }

    pub fn on_tick(&mut self) {
        // Handle tick
    }

    pub fn render(&mut self, frame: &mut Frame) {
        crate::ui::render::render(frame, self);
    }
}
