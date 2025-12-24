mod app;
mod config;
mod event;
mod error;
mod ui;
mod aws;
mod models;
mod utils;

use app::{App, Message};
use event::{Event, EventHandler};
use ratatui::{backend::CrosstermBackend, Terminal};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::sync::{Arc, atomic::AtomicBool};

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

    // Determine the actual profile and region being used
    let profile = args.profile.clone()
        .or_else(|| std::env::var("AWS_PROFILE").ok());
    let region = args.region.clone()
        .or_else(|| std::env::var("AWS_REGION").ok())
        .or_else(|| std::env::var("AWS_DEFAULT_REGION").ok())
        .unwrap_or_else(|| "us-east-1".to_string());

    // Create shared input pause flag
    let input_paused = Arc::new(AtomicBool::new(false));

    // Create app state
    let mut app = App::new(aws_clients, profile.clone(), region, args.read_only, input_paused.clone());

    // Create event handler
    let mut events = EventHandler::new(app.config.tick_rate_ms, input_paused);
    let event_tx = events.sender();

    // If no profile was specified, open the profile switcher immediately
    if profile.is_none() {
        app.update(Message::open_profile_switcher(), event_tx.clone());
    } else {
        // Initial data load
        app.update(Message::refresh(), event_tx.clone());
    }

    // Main loop
    while !app.should_quit {
        // Check if we need to force a terminal redraw (e.g., after external editor)
        if app.needs_redraw {
            app.needs_redraw = false;
            terminal.clear()?;
        }
        
        // Render
        terminal.draw(|frame| app.render(frame))?;

        // Handle events
        if let Some(event) = events.next().await {
            match event {
                Event::Key(key) => {
                    // Handle Ctrl-C to quit
                    if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) 
                        && key.code == crossterm::event::KeyCode::Char('c') 
                    {
                        break;
                    }
                    if let Some(msg) = app.handle_key(key) {
                        app.update(msg, event_tx.clone());
                    }
                }
                Event::Tick => {
                    app.on_tick();
                    if app.should_refresh {
                        app.should_refresh = false;
                        app.update(Message::refresh(), event_tx.clone());
                    }
                }
                Event::Aws(aws_event) => {
                    app.handle_aws_event(*aws_event);
                    
                    // Check for pending S3 edits that need synchronous processing
                    if let Some((bucket, key, path)) = app.services.s3.pending_edit.take() {
                        app.handle_edit_s3_object_sync(bucket, key, path, event_tx.clone());
                    }
                    
                    // Check for pending ECS task definition edits that need synchronous processing
                    if let Some((family, path)) = app.services.ecs.pending_edit.take() {
                        app.handle_edit_task_definition_sync(family, path, event_tx.clone());
                    }
                }
                Event::Message(msg) => {
                    app.update(msg, event_tx.clone());
                }
            }
        }
    }

    // Cleanup: cancel all pending async tasks
    app.shutdown();

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}
