use std::{io, time::Duration, sync::mpsc, fs::OpenOptions};
use clap::Parser;
use cli::{Cli, LogLevel};
use tui::{
    backend::CrosstermBackend,
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[macro_use]
extern crate log;
use simplelog::*;

use std::fs::File;

mod app;
mod cli;
mod ui;
use app::{App, UiWidget, ScrollEvent};

pub enum IoEvent {
    Quit,
    Tick,
    ToggleWidget(UiWidget),
    Scroll(ScrollEvent),
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();
    setup_logging(cli.log_level, cli.log_file);
    let (tx, rx) = mpsc::channel();
    let app = App::new(tx.clone());

    // Enable debug widget
    app.enable_widget(UiWidget::Debug);

    // Add some dummy nodes
    {
        app.add_node(123, cli.ring);
    }

    let mut stdout = io::stdout();
    enable_raw_mode()?;

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut tick = tokio::time::interval(Duration::from_millis(100));

    loop {
        tick.tick().await;

        let app = app.clone();
        terminal.draw(|f| {
            ui::render_home(f, app.clone());
        })?;

        if event::poll(Duration::from_millis(100)).expect("poll works") {
            if let event::Event::Key(key) = event::read().expect("can read events") {
                if key.code == event::KeyCode::Char('q') {
                    tx.send(IoEvent::Quit).unwrap();
                }

                if key.code == event::KeyCode::Char('d') {
                    tx.send(IoEvent::ToggleWidget(UiWidget::Debug)).unwrap();
                }

                if key.code == event::KeyCode::Char('/') {
                    tx.send(IoEvent::ToggleWidget(UiWidget::Search)).unwrap();
                }

                if key.code == event::KeyCode::Char('j') {
                    tx.send(IoEvent::Scroll(ScrollEvent::Down)).unwrap();
                }

                if key.code == event::KeyCode::Char('k') {
                    tx.send(IoEvent::Scroll(ScrollEvent::Up)).unwrap();
                }
            }
        }

        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(IoEvent::Quit) => {
                break;
            }
            Ok(IoEvent::Tick) => {
            }

            Ok(IoEvent::ToggleWidget(widget)) => {
                if app.widget_enabled(widget) {
                    app.disable_widget(widget);
                } else {
                    app.enable_widget(widget);
                }
            }

            Ok(IoEvent::Scroll(event)) => {
                app.scroll(event);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

// #[tokio::main]
// async fn start(app: App, io_rx: std::sync::mpsc::Receiver<IoEvent>) -> Result<(), io::Error> {

//     Ok(())
// }

fn setup_logging(log_level: LogLevel, log_file: Option<String>) {
    let mut config = ConfigBuilder::new();
    config.set_time_format_rfc3339();
    let config = config.build();


    let logger: Box<dyn SharedLogger> = if let Some(log_file) = log_file {
        let output_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file);
        if let Err(e) = output_file {
            panic!("Failed to open log file: {}", e);
        }
        WriteLogger::new(log_level.into(), config.clone(), output_file.unwrap())
    } else {
        TermLogger::new(log_level.into(), config.clone(), TerminalMode::Stderr, ColorChoice::Auto)
    };

    CombinedLogger::init(vec![logger]).unwrap();

    info!("Logging started");
}
