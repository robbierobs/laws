mod app;
mod config;
mod event;
mod ui;
mod aws;
mod models;
mod actions;
mod utils;

use app::{App, Message};
use event::{Event, EventHandler};
use ratatui::{backend::CrosstermBackend, Terminal};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

use crate::aws::client::AwsClients;
use crate::config::Args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let args = Args::parse_args();
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize AWS clients
    // Priority: CLI args > environment variables > defaults
    let aws_clients = match AwsClients::new(
        args.profile.as_deref(),
        args.region.as_deref(),
        args.endpoint_url.as_deref(),
    ).await {
        Ok(clients) => Some(clients),
        Err(e) => {
            eprintln!("Failed to initialize AWS clients: {}", e);
            None
        }
    };

    // Create app state
    let mut app = App::new(aws_clients);

    // Create event handler
    let mut events = EventHandler::new(250); // 250ms tick rate
    let event_tx = events.sender();

    // Initial data load
    app.update(Message::RefreshData, event_tx.clone()).await;

    // Main loop
    while !app.should_quit {
        // Render
        terminal.draw(|frame| app.render(frame))?;

        // Handle events
        if let Some(event) = events.next().await {
            match event {
                Event::Key(key) => {
                    if let Some(msg) = app.handle_key(key) {
                        app.update(msg, event_tx.clone()).await;
                    }
                }
                Event::Tick => {
                    app.on_tick();
                    if app.should_refresh {
                        app.should_refresh = false;
                        app.update(Message::RefreshData, event_tx.clone()).await;
                    }
                }
                Event::Aws(aws_event) => {
                    app.handle_aws_event(aws_event);
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}
