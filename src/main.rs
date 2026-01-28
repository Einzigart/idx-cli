mod api;
mod app;
mod config;
mod ui;

use anyhow::Result;
use app::{App, InputMode, ViewMode};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;
use tokio::time::Instant;

#[derive(Parser)]
#[command(name = "idx-cli")]
#[command(about = "Terminal UI for Indonesian stock market data", long_about = None)]
struct Cli {
    /// Refresh interval in seconds
    #[arg(short, long, default_value = "5")]
    interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new()?;
    app.config.refresh_interval_secs = cli.interval;

    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let refresh_interval = Duration::from_secs(app.config.refresh_interval_secs);
    let mut last_refresh = Instant::now() - refresh_interval; // Force immediate refresh

    loop {
        // Auto-refresh quotes
        if last_refresh.elapsed() >= refresh_interval {
            app.refresh_quotes().await?;
            last_refresh = Instant::now();
        }

        // Draw UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle input with timeout for refresh
        // Use 100ms timeout to keep clock updating smoothly
        let timeout = Duration::from_millis(100);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('?') => app.show_help(),
                        KeyCode::Char('/') => app.start_search(),
                        KeyCode::Char('e') => app.start_export(),
                        KeyCode::Char('p') => {
                            app.toggle_view();
                            app.refresh_quotes().await?;
                            last_refresh = Instant::now();
                        }
                        KeyCode::Char('a') => {
                            match app.view_mode {
                                ViewMode::Watchlist => app.start_adding(),
                                ViewMode::Portfolio => app.start_portfolio_add(),
                            }
                        }
                        KeyCode::Char('d') => {
                            match app.view_mode {
                                ViewMode::Watchlist => app.remove_selected()?,
                                ViewMode::Portfolio => app.remove_selected_holding()?,
                            }
                            app.refresh_quotes().await?;
                            last_refresh = Instant::now();
                        }
                        KeyCode::Char('r') => {
                            app.refresh_quotes().await?;
                            last_refresh = Instant::now();
                        }
                        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                        KeyCode::Left | KeyCode::Char('h') => {
                            if app.view_mode == ViewMode::Watchlist {
                                app.prev_watchlist();
                                app.refresh_quotes().await?;
                                last_refresh = Instant::now();
                            }
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            if app.view_mode == ViewMode::Watchlist {
                                app.next_watchlist();
                                app.refresh_quotes().await?;
                                last_refresh = Instant::now();
                            }
                        }
                        KeyCode::Char('n') => {
                            if app.view_mode == ViewMode::Watchlist {
                                app.start_watchlist_add();
                            }
                        }
                        KeyCode::Char('R') => {
                            if app.view_mode == ViewMode::Watchlist {
                                app.start_watchlist_rename();
                            }
                        }
                        KeyCode::Char('D') => {
                            if app.view_mode == ViewMode::Watchlist {
                                app.remove_current_watchlist()?;
                                app.refresh_quotes().await?;
                                last_refresh = Instant::now();
                            }
                        }
                        KeyCode::Enter => {
                            match app.view_mode {
                                ViewMode::Watchlist => app.show_stock_detail().await,
                                ViewMode::Portfolio => app.show_portfolio_detail().await,
                            }
                        }
                        _ => {}
                    },
                    InputMode::Adding => match key.code {
                        KeyCode::Enter => {
                            app.confirm_add()?;
                            app.refresh_quotes().await?;
                            last_refresh = Instant::now();
                        }
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            if c.is_alphanumeric() {
                                app.input_buffer.push(c);
                            }
                        }
                        _ => {}
                    },
                    InputMode::WatchlistAdd | InputMode::WatchlistRename => match key.code {
                        KeyCode::Enter => {
                            if app.input_mode == InputMode::WatchlistAdd {
                                app.confirm_watchlist_add()?;
                            } else {
                                app.confirm_watchlist_rename()?;
                            }
                            app.refresh_quotes().await?;
                            last_refresh = Instant::now();
                        }
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            // Allow alphanumeric and spaces for watchlist names
                            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                                app.input_buffer.push(c);
                            }
                        }
                        _ => {}
                    },
                    InputMode::PortfolioAdd => match key.code {
                        KeyCode::Enter => {
                            app.confirm_portfolio_add()?;
                            app.refresh_quotes().await?;
                            last_refresh = Instant::now();
                        }
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            // Allow alphanumeric, comma, and dot for portfolio input
                            if c.is_alphanumeric() || c == ',' || c == '.' {
                                app.input_buffer.push(c);
                            }
                        }
                        _ => {}
                    },
                    InputMode::StockDetail => match key.code {
                        KeyCode::Esc | KeyCode::Enter => app.close_stock_detail(),
                        _ => {}
                    },
                    InputMode::Help => match key.code {
                        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?') => app.close_help(),
                        _ => {}
                    },
                    InputMode::Search => match key.code {
                        KeyCode::Enter => app.confirm_search(),
                        KeyCode::Esc => app.cancel_search(),
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            app.input_buffer.push(c);
                        }
                        _ => {}
                    },
                    InputMode::ExportMenu => match key.code {
                        KeyCode::Esc => app.cancel_export(),
                        KeyCode::Up | KeyCode::Char('k') => app.export_menu_up(),
                        KeyCode::Down | KeyCode::Char('j') => app.export_menu_down(),
                        KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                            match app.export_menu_selection {
                                0 => app.toggle_export_format(),
                                1 => app.toggle_export_scope(),
                                _ => {}
                            }
                        }
                        KeyCode::Enter => app.confirm_export()?,
                        _ => {}
                    },
                }
            }
        }
    }
}
