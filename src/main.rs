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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Create event handler
    let mut events = EventHandler::new(250); // 250ms tick rate
    let event_tx = events.sender();

    // Main loop
    while !app.should_quit {
        // Render
        terminal.draw(|frame| app.render(frame))?;

        // Handle events
        if let Some(event) = events.next().await {
            match event {
                Event::Key(key) => {
                    if key.code == crossterm::event::KeyCode::Char('q') {
                        app.update(Message::Quit, event_tx.clone()).await;
                    }
                }
                Event::Tick => {
                    app.on_tick();
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
