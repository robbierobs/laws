//! Generic instance action handler
//!
//! Provides a trait-based abstraction for services that support
//! start/stop/reboot operations (EC2, RDS, etc.)

use crate::error::AppResult;
use crate::event::{AwsEvent, Event};
use std::future::Future;
use std::pin::Pin;

/// Trait for services that support instance lifecycle actions (start/stop/reboot)
pub trait InstanceActionService: Send + Sync + 'static {
    /// The service name for display (e.g., "EC2", "RDS")
    fn service_name(&self) -> &'static str;
    
    /// Start an instance
    fn start<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>;
    
    /// Stop an instance
    fn stop<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>;
    
    /// Reboot an instance
    fn reboot<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>;

    /// Terminate/Delete an instance
    fn terminate<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>;
}

/// Execute an instance action (start/stop/reboot) and send appropriate events
pub async fn execute_instance_action<S: InstanceActionService>(
    service: S,
    action: &str,
    id: String,
    tx: crate::app::EventSender,
) {
    let result = match action {
        "start" => service.start(&id).await,
        "stop" => service.stop(&id).await,

        "reboot" => service.reboot(&id).await,
        "terminate" => service.terminate(&id).await,
        _ => return,
    };

    let service_name = service.service_name();
    match result {
        Ok(_) => {
            let action_past = format!("{}ed", action.trim_end_matches('e'));
            let msg = format!("{} {} instance {}", action_past, service_name, id);
            tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(msg)))).await.ok();
        }
        Err(e) => {
            tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
        }
    }
}

// ============================================================================
// Implementations for AWS services
// ============================================================================

impl InstanceActionService for crate::aws::ec2::Ec2Service {
    fn service_name(&self) -> &'static str {
        "EC2"
    }
    
    fn start<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.start_instance(id))
    }
    
    fn stop<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.stop_instance(id))
    }
    
    fn reboot<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.reboot_instance(id))
    }

    fn terminate<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.terminate_instance(id))
    }
}

impl InstanceActionService for crate::aws::rds::RdsService {
    fn service_name(&self) -> &'static str {
        "RDS"
    }
    
    fn start<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.start_instance(id))
    }
    
    fn stop<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.stop_instance(id))
    }
    
    fn reboot<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.reboot_instance(id))
    }

    fn terminate<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.delete_instance(id))
    }
}
