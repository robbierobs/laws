use crate::app::states::ServiceInternal;
use crate::app::task_manager::{task_keys, TaskManager};
use crate::app::update::refresh::spawn_list_task;
use crate::app::{EventSender, InputResult, Message, ServiceInputHandler, TableStateExt};
use crate::aws::client::AwsClients;
use crate::event::AwsEvent;
use crate::models::sqs::SqsQueue;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;

#[derive(Default)]
pub struct SqsState {
    pub queues: Vec<SqsQueue>,
    pub list_state: TableState,
}

impl SqsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn selected_queue(&self) -> Option<&SqsQueue> {
        self.list_state.selected().and_then(|i| self.queues.get(i))
    }
}

impl crate::app::global_search::Searchable for SqsState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::global_search::SearchResult;
        use crate::app::Service;

        self.queues
            .iter()
            .map(|q| {
                let mut result = SearchResult::new(Service::SQS, "SQS Queue", &q.queue_name);
                if q.is_fifo {
                    result = result.with_secondary("FIFO".to_string());
                }
                result
            })
            .collect()
    }
}

impl crate::app::global_search::AutoSelectable for SqsState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        if let Some(idx) = self.queues.iter().position(|q| q.queue_name == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

impl ServiceInputHandler for SqsState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.nav_down(self.queues.len());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.nav_up(self.queues.len());
            }
            KeyCode::Char('P') => {
                if let Some(q) = self.selected_queue() {
                    return InputResult::Action(Message::sqs_purge(q.queue_url.clone()));
                }
            }
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(q) = self.selected_queue() {
                    return InputResult::Action(Message::sqs_delete(q.queue_url.clone()));
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }

    fn get_copiable_text(&self) -> Option<String> {
        self.selected_queue().map(|q| q.queue_url.clone())
    }
}

impl ServiceInternal for SqsState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.sqs.clone();
        let handle = spawn_list_task(
            tx,
            move || async move { crate::aws::sqs::SqsService::new(client).list_queues().await },
            AwsEvent::SqsQueuesLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::SQS_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.queues.clear();
        self.list_state.select(Some(0));
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() && !self.queues.is_empty() {
            self.list_state.select(Some(0));
        }
    }
}
