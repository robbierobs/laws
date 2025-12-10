use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum Event {
    Key(KeyEvent),
    Tick,
    Aws(AwsEvent),
}

use crate::models::ec2::Ec2Instance;
use crate::models::s3::S3Bucket;

#[derive(Debug)]
pub enum AwsEvent {
    Ec2InstancesLoaded(Vec<Ec2Instance>),
    S3BucketsLoaded(Vec<S3Bucket>),
    ActionCompleted(String), // Message to display
    Error(String),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        // Spawn input handling task
        tokio::spawn(async move {
            let tick_rate = Duration::from_millis(tick_rate);
            loop {
                let event_available = event::poll(tick_rate).unwrap_or(false);
                if event_available {
                    if let Ok(CrosstermEvent::Key(key)) = event::read() {
                        event_tx.send(Event::Key(key)).unwrap_or(());
                    }
                }
                event_tx.send(Event::Tick).ok();
            }
        });

        Self { rx, _tx: tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self._tx.clone()
    }
}
