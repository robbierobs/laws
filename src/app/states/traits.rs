use crate::app::EventSender;
use crate::aws::client::AwsClients;
use crate::app::task_manager::TaskManager;
use crate::app::global_search::{AutoSelectable, Searchable};

/// Internal trait for service states to handle system operations
/// 
/// This trait standardizes how the main application interacts with 
/// service-specific states for background tasks and data management.
pub trait ServiceInternal: AutoSelectable + Searchable + Send {
    /// Refresh the service's primary data
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        config: &crate::config::AppConfig,
        report_errors: bool,
    );
    
    /// Clear the service's current data (e.g. on profile switch)
    fn clear(&mut self);
    
    /// Auto-select the first item if nothing is selected
    /// 
    /// Each service implements this based on its current view mode
    /// and data availability.
    fn auto_select_first(&mut self);
}
