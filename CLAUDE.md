# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

idx-cli is a terminal UI (TUI) application for tracking Indonesian stock market (IDX) data. It provides real-time stock quotes, multiple watchlists, and portfolio tracking with P/L calculations.

## Tech Stack

- **Language**: Rust (Edition 2024)
- **TUI Framework**: ratatui + crossterm
- **Async Runtime**: tokio
- **HTTP Client**: reqwest (with cookie support for Yahoo Finance authentication)
- **Serialization**: serde + serde_json
- **CLI**: clap (derive feature)

## Commands

```bash
cargo build              # Build debug
cargo build --release    # Build release
cargo run                # Run with default 5s refresh
cargo run -- -i 10       # Run with custom refresh interval (seconds)
cargo test               # Run tests
cargo clippy             # Lint
cargo fmt                # Format code
```

## Architecture

```
src/
├── main.rs      # Entry point, terminal setup, event loop, keybindings
├── app.rs       # Application state (App struct), business logic, mode management
├── ui.rs        # All rendering: watchlist table, portfolio table, stock detail popup
├── config.rs    # Persistent config: watchlists, holdings, JSON file storage
└── api/
    ├── mod.rs   # Re-exports
    └── yahoo.rs # Yahoo Finance API client with crumb authentication
```

### Key Patterns

**State Machine**: `InputMode` enum controls UI behavior (Normal, Adding, WatchlistAdd, WatchlistRename, StockDetail, PortfolioAdd). `ViewMode` toggles between Watchlist and Portfolio views.

**Data Flow**: `App` holds `Config` (persistent), `quotes` cache (volatile), and `YahooClient`. Quote refresh is triggered by timer or user action. Config saves to `~/.config/idx-cli/config.json`.

**Yahoo Finance Auth**: The `YahooClient` fetches a crumb token from the main page and uses cookies for authenticated API requests. Crumb refresh happens on 401 responses.

**IDX Symbol Convention**: Stock codes (e.g., "BBCA") are internally converted to Yahoo format ("BBCA.JK") for API calls, then stripped back for display.

## Keybindings (Normal Mode)

| Key | Watchlist | Portfolio |
|-----|-----------|-----------|
| `p` | Switch to Portfolio | Switch to Watchlist |
| `a` | Add stock | Add holding (SYMBOL,LOTS,PRICE) |
| `d` | Delete selected | Delete selected |
| `r` | Refresh quotes | Refresh quotes |
| `Enter` | Stock detail popup | Stock detail popup |
| `j/k` or `↑/↓` | Navigate | Navigate |
| `h/l` or `←/→` | Prev/Next watchlist | - |
| `n` | New watchlist | - |
| `R` | Rename watchlist | - |
| `D` | Delete watchlist | - |
| `q` | Quit | Quit |
