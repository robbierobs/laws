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
use crate::error::classify_credential_error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let args = Args::parse_args();

    // Determine profile early — needed for SSO login before TUI starts
    let profile = args.profile.clone()
        .or_else(|| std::env::var("AWS_PROFILE").ok());

    // If an SSO profile is specified, ensure credentials are valid before entering TUI.
    // This runs before enable_raw_mode() so the user can see SSO browser prompts.
    if let Some(ref profile_name) = profile {
        if crate::utils::aws_profiles::is_sso_profile(profile_name) {
            try_sso_login_at_startup(profile_name).await;
        }
    }
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize AWS clients
    // Priority: CLI args > environment variables > defaults
    let mut init_error: Option<String> = None;
    let aws_clients = match AwsClients::new(
        args.profile.as_deref(),
        args.region.as_deref(),
        args.endpoint_url.as_deref(),
    ).await {
        Ok(clients) => Some(clients),
        Err(e) => {
            init_error = Some(e.to_string());
            None
        }
    };

    let mut startup_error: Option<String> = None;
    if let Some(clients) = &aws_clients {
        if let Err(e) = clients.validate().await {
            let message = e.to_string();
            startup_error = Some(
                classify_credential_error(&message)
                    .map(|err| err.user_message())
                    .unwrap_or_else(|| e.user_message()),
            );
        }
    } else {
        let message = init_error
            .unwrap_or_else(|| "Failed to initialize AWS clients".to_string());
        startup_error = Some(
            classify_credential_error(&message)
                .map(|err| err.user_message())
                .unwrap_or(message),
        );
    }

    // Determine the actual region being used
    let region = args.region.clone()
        .or_else(|| std::env::var("AWS_REGION").ok())
        .or_else(|| std::env::var("AWS_DEFAULT_REGION").ok())
        .unwrap_or_else(|| "us-east-1".to_string());

    // Create shared input pause flag
    let input_paused = Arc::new(AtomicBool::new(false));

    // Create app state
    let mut app = App::new(
        aws_clients,
        profile.clone(),
        region,
        args.read_only,
        input_paused.clone(),
    );
    if let Some(message) = startup_error {
        app.error_message = Some(message.clone());
        app.action_log.push(format!("[ERROR] {}", message));
    }

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

/// Attempt SSO login before the TUI starts.
///
/// Runs before `enable_raw_mode()` so the user sees SSO browser prompts
/// and URLs in their normal terminal. Non-fatal: if anything fails, the
/// TUI will still start and show the credential error via `clients.validate()`.
async fn try_sso_login_at_startup(profile_name: &str) {
    use std::time::Duration;

    let config = crate::config::ConfigFile::load();
    let timeout_secs = config.sso_login_timeout_secs;

    // Check if AWS CLI is available
    let cli_check = tokio::process::Command::new("aws")
        .arg("--version")
        .kill_on_drop(true)
        .output()
        .await;

    let cli_available = cli_check
        .as_ref()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !cli_available {
        // No AWS CLI — can't do SSO login. The TUI will show the credential error.
        return;
    }

    // Check if existing SSO credentials are still valid
    let creds_check = tokio::process::Command::new("aws")
        .args(["sts", "get-caller-identity", "--profile", profile_name])
        .kill_on_drop(true)
        .output()
        .await;

    let needs_login = match creds_check {
        Ok(output) => !output.status.success(),
        Err(_) => true,
    };

    if !needs_login {
        return;
    }

    // Credentials expired or missing — run SSO login
    eprintln!("SSO credentials expired for profile '{}'. Launching SSO login...", profile_name);

    let mut command = tokio::process::Command::new("aws");
    command
        .args(["sso", "login", "--profile", profile_name])
        .kill_on_drop(true)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    let sso_result = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        command.status(),
    )
    .await;

    match sso_result {
        Ok(Ok(status)) => {
            if status.success() {
                eprintln!("SSO login successful.");
            } else {
                eprintln!("SSO login exited with status: {}. Continuing to TUI...", status);
            }
        }
        Ok(Err(e)) => {
            eprintln!("SSO login failed: {}. Continuing to TUI...", e);
        }
        Err(_) => {
            eprintln!(
                "SSO login timed out after {}s. Complete login in the browser, then refresh in the TUI.",
                timeout_secs
            );
        }
    }
}
