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
    #[arg(short, long, default_value = "1")]
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

/// Draw, fetch quotes, and reset the refresh timer.
/// The caller must have already called `app.prepare_refresh()` so that
/// `loading = true` is visible in the draw that happens here.
async fn refresh_and_draw<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    symbols: &[String],
    last_refresh: &mut Instant,
) -> Result<()> {
    terminal.draw(|frame| ui::draw(frame, app))?;
    app.execute_refresh(symbols).await?;
    *last_refresh = Instant::now();
    Ok(())
}

/// Draw, fetch news feeds, then clear the loading flag.
async fn refresh_news_and_draw<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    urls: &[String],
) -> Result<()> {
    terminal.draw(|frame| ui::draw(frame, app))?;
    app.execute_news_refresh(urls).await;
    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let refresh_interval = Duration::from_secs(app.config.refresh_interval_secs);
    let news_refresh_interval = Duration::from_secs(300); // 5 minutes
    let mut last_refresh = Instant::now() - refresh_interval; // Force immediate refresh

    let urls = app.prepare_news_refresh();
    refresh_news_and_draw(terminal, app, &urls).await?;

    loop {
        // Auto-refresh quotes (skip in News view)
        if app.view_mode != ViewMode::News
            && last_refresh.elapsed() >= refresh_interval
            && let Some(symbols) = app.prepare_refresh()
        {
            refresh_and_draw(terminal, app, &symbols, &mut last_refresh).await?;
        }

        // Auto-refresh news when in News view
        if app.view_mode == ViewMode::News {
            let should_refresh = match app.news_last_refresh {
                Some(last) => last.elapsed() >= news_refresh_interval,
                None => true,
            };
            if should_refresh && !app.rss_loading {
                let urls = app.prepare_news_refresh();
                refresh_news_and_draw(terminal, app, &urls).await?;
            }
        }

        // Draw UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle input with timeout for refresh
        // Use 100ms timeout to keep clock updating smoothly
        let timeout = Duration::from_millis(100);

        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            let mut needs_refresh = false;

            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('?') => app.show_help(),
                    KeyCode::Char('/') => app.start_search(),
                    KeyCode::Char('e') => {
                        match app.view_mode {
                            ViewMode::Portfolio => app.start_portfolio_edit(),
                            ViewMode::Watchlist => app.start_export(),
                            ViewMode::News => {}
                        }
                    }
                    KeyCode::Char('p') => {
                        app.toggle_view();
                        if app.view_mode == ViewMode::News {
                            if app.news_last_refresh.is_none() {
                                let urls = app.prepare_news_refresh();
                                refresh_news_and_draw(terminal, app, &urls).await?;
                            }
                        } else {
                            needs_refresh = true;
                        }
                    }
                    KeyCode::Char('a') => {
                        match app.view_mode {
                            ViewMode::Watchlist => app.start_adding(),
                            ViewMode::Portfolio => app.start_portfolio_add(),
                            ViewMode::News => {}
                        }
                    }
                    KeyCode::Char('d') => {
                        match app.view_mode {
                            ViewMode::Watchlist => app.remove_selected()?,
                            ViewMode::Portfolio => app.remove_selected_holding()?,
                            ViewMode::News => {}
                        }
                        needs_refresh = true;
                    }
                    KeyCode::Char('r') => {
                        if app.view_mode == ViewMode::News {
                            let urls = app.prepare_news_refresh();
                            refresh_news_and_draw(terminal, app, &urls).await?;
                        } else {
                            needs_refresh = true;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Left | KeyCode::Char('h') => {
                        if app.view_mode == ViewMode::Watchlist {
                            app.prev_watchlist();
                            needs_refresh = true;
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if app.view_mode == ViewMode::Watchlist {
                            app.next_watchlist();
                            needs_refresh = true;
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
                            needs_refresh = true;
                        }
                    }
                    KeyCode::Enter => {
                        match app.view_mode {
                            ViewMode::Watchlist => app.show_stock_detail().await,
                            ViewMode::Portfolio => app.show_portfolio_detail().await,
                            ViewMode::News => {}
                        }
                    }
                    KeyCode::Char('s') => app.cycle_sort_column(),
                    KeyCode::Char('S') => app.toggle_sort_direction(),
                    KeyCode::Char('c') => {
                        if app.view_mode == ViewMode::Portfolio {
                            app.show_portfolio_chart();
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
                InputMode::PortfolioChart => match key.code {
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char('c') => app.close_portfolio_chart(),
                    _ => {}
                },
                // All text-input modes share common Backspace/Esc handling
                _ => match key.code {
                    KeyCode::Esc => {
                        match app.input_mode {
                            InputMode::PortfolioAddSymbol
                            | InputMode::PortfolioAddLots
                            | InputMode::PortfolioAddPrice => app.cancel_portfolio_add(),
                            InputMode::PortfolioEditLots
                            | InputMode::PortfolioEditPrice => app.cancel_portfolio_edit(),
                            InputMode::Search => app.cancel_search(),
                            _ => app.cancel_input(),
                        }
                    }
                    KeyCode::Enter => {
                        match app.input_mode {
                            InputMode::Adding => {
                                app.confirm_add()?;
                                needs_refresh = true;
                            }
                            InputMode::WatchlistAdd => {
                                app.confirm_watchlist_add()?;
                                needs_refresh = true;
                            }
                            InputMode::WatchlistRename => {
                                app.confirm_watchlist_rename()?;
                                needs_refresh = true;
                            }
                            InputMode::PortfolioAddSymbol => app.confirm_portfolio_symbol(),
                            InputMode::PortfolioAddLots => app.confirm_portfolio_lots(),
                            InputMode::PortfolioAddPrice => {
                                app.confirm_portfolio_price()?;
                                needs_refresh = true;
                            }
                            InputMode::PortfolioEditLots => app.confirm_portfolio_edit_lots(),
                            InputMode::PortfolioEditPrice => {
                                app.confirm_portfolio_edit_price()?;
                                needs_refresh = true;
                            }
                            InputMode::Search => app.confirm_search(),
                            _ => {}
                        }
                    }
                    KeyCode::Backspace => { app.input_buffer.pop(); }
                    KeyCode::Char(c) => {
                        let allowed = match app.input_mode {
                            InputMode::Adding | InputMode::PortfolioAddSymbol => c.is_alphanumeric(),
                            InputMode::PortfolioAddLots | InputMode::PortfolioEditLots => c.is_ascii_digit(),
                            InputMode::PortfolioAddPrice | InputMode::PortfolioEditPrice => c.is_ascii_digit() || c == '.',
                            InputMode::WatchlistAdd | InputMode::WatchlistRename => {
                                c.is_alphanumeric() || c == ' ' || c == '-' || c == '_'
                            }
                            InputMode::Search => true,
                            _ => false,
                        };
                        if allowed {
                            app.input_buffer.push(c);
                        }
                    }
                    _ => {}
                },
            }

            if needs_refresh
                && let Some(symbols) = app.prepare_refresh()
            {
                refresh_and_draw(terminal, app, &symbols, &mut last_refresh).await?;
            }
        }
    }
}