# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

idx-cli is a terminal UI (TUI) application for tracking Indonesian stock market (IDX) data. It provides real-time stock quotes, IHSG composite index display, multiple watchlists, multiple portfolios with P/L calculations.

## Tech Stack

- **Language**: Rust (Edition 2024)
- **TUI Framework**: ratatui + crossterm
- **Async Runtime**: tokio
- **HTTP Client**: reqwest (with cookie support for Yahoo Finance authentication)
- **RSS Parsing**: feed-rs (auto-detects RSS/Atom/JSON Feed)
- **Async Utilities**: futures (for concurrent feed fetching)
- **Serialization**: serde + serde_json
- **CLI**: clap (derive feature)

## Commands

```bash
cargo build              # Build debug
cargo build --release    # Build release
cargo run                # Run with default 1s refresh
cargo run -- -i 10       # Run with custom refresh interval (seconds)
cargo test               # Run tests
cargo clippy             # Lint
cargo fmt                # Format code
```

## Architecture

```
src/
├── main.rs          # Entry point, terminal setup, event loop, keybindings
├── app/
│   ├── mod.rs       # App struct, enums, navigation, sort, core helpers
│   ├── watchlist.rs # Watchlist CRUD: add/remove stocks, watchlist management
│   ├── portfolio.rs # Portfolio CRUD: add wizard, remove, edit, detail, chart, multi-portfolio nav
│   ├── news.rs      # RSS news: refresh_news() fetches aggregated feed headlines
│   ├── export.rs    # Export menu logic, CSV/JSON formatters
│   ├── filter.rs    # Search/filter, get_filtered_watchlist/portfolio/news, selected_*_symbol
│   └── sort.rs      # Column comparison functions for watchlist, portfolio, and news sorting
├── ui/
│   ├── mod.rs       # draw() entry point, draw_header(), draw_footer()
│   ├── tables.rs    # Column defs, visible_columns(), draw_watchlist(), draw_portfolio()
│   ├── news.rs      # draw_news() table for RSS headlines (Time, Source, Headline columns)
│   ├── modals.rs    # draw_help(), draw_export_menu(), draw_portfolio_chart()
│   ├── news_detail.rs # draw_news_detail() modal with HTML stripping, word wrap, scrollable preview
│   ├── detail.rs    # draw_stock_detail() with section helpers (price, range, fundamentals, risk)
│   └── formatters.rs # format_price, format_change, format_compact, format_pl, format_relative_time, etc.
├── config.rs        # Persistent config: watchlists, portfolios, holdings, JSON file storage + migration
└── api/
    ├── mod.rs       # Re-exports
    ├── yahoo.rs     # Yahoo Finance API client with crumb authentication
    └── news.rs      # RSS feed client: NewsClient fetches/parses feeds via feed-rs
```

### Key Patterns

**State Machine**: `InputMode` enum controls UI behavior (Normal, Adding, WatchlistAdd, WatchlistRename, StockDetail, PortfolioAddSymbol/Lots/Price, PortfolioNew, PortfolioRename, Help, Search, ExportMenu). `ViewMode` cycles between Watchlist, Portfolio, and News views.

**Data Flow**: `App` holds `Config` (persistent), `quotes` cache (volatile), `YahooClient`, and `NewsClient`. Quote refresh is triggered by timer or user action and always includes `^JKSE` (IHSG composite index) for the header bar. News refresh has a separate 5-minute timer and uses `NewsClient` with concurrent feed fetching via `futures::join_all`. Config saves to `~/.config/idx-cli/config.json`.

**Yahoo Finance Auth**: The `YahooClient` fetches a crumb token from the main page and uses cookies for authenticated API requests. Crumb refresh happens on 401 responses.

**Multi-Step Input**: Portfolio add uses a wizard flow (Symbol → Lots → Price) with `pending_symbol`/`pending_lots` fields on `App`. Each step validates independently. `cancel_portfolio_add()` resets all pending state. Prefer this pattern over comma-separated single-line input for multi-field forms.

**IDX Symbol Convention**: Stock codes (e.g., "BBCA") are internally converted to Yahoo format ("BBCA.JK") for API calls, then stripped back for display. Index symbols (e.g., "^JKSE") are passed through unchanged; `^JKSE` displays as `IHSG`.

**Responsive Columns**: `ColumnDef` structs with priority tiers (1=always visible, 4=only on wide terminals). `visible_columns()` greedily includes columns from highest to lowest priority until available width is exhausted. Stretch columns (Name for watchlist, Value for portfolio) absorb extra space via `Constraint::Min()`.

**Column Sorting**: `SortDirection` enum with `cycle_sort_column()` / `toggle_sort_direction()`. Sort applied in `get_filtered_watchlist()` / `get_filtered_portfolio()` / `get_filtered_news()` after search filtering. Free functions `compare_watchlist_column()` / `compare_portfolio_column()` / `compare_news_column()` handle per-column type-aware comparison. Export functions use `get_raw_watchlist()` (insertion order, not sorted).

**Selection via Filtered View**: `selected_index` / `portfolio_selected` / `news_selected` are indices into the **filtered/sorted** list, not the raw list. All operations (delete, detail view, navigation bounds) must resolve through the filtered list helpers.

## Keybindings (Normal Mode)

| Key | Watchlist | Portfolio | News |
|-----|-----------|-----------|------|
| `p` | → Portfolio | → News | → Watchlist |
| `a` | Add stock | Add holding (step-by-step) | - |
| `d` | Delete selected | Delete selected | - |
| `r` | Refresh quotes | Refresh quotes | Refresh feeds |
| `Enter` | Stock detail popup | Stock detail popup | News detail preview |
| `j/k` or `↑/↓` | Navigate | Navigate | Navigate |
| `h/l` or `←/→` | Prev/Next watchlist | Prev/Next portfolio | - |
| `n` | New watchlist | New portfolio | - |
| `R` | Rename watchlist | Rename portfolio | - |
| `D` | Delete watchlist | Delete portfolio | - |
| `s` | Cycle sort column | Cycle sort column | Cycle sort column |
| `S` | Toggle sort direction | Toggle sort direction | Toggle sort direction |
| `/` | Search symbols | Search symbols | Search headlines |
| `c` | - | Portfolio allocation chart | - |
| `e` | Export data | Export data | - |
| `q` | Quit | Quit | Quit |

## Config File

Stored at `~/.config/idx-cli/config.json`. Schema managed by `Config` struct in `config.rs`. Contains:
- `watchlists`: Vec of `{name, symbols: Vec<String>}` with `active_watchlist` index
- `portfolios`: Vec of `{name, holdings: Vec<{symbol, lots: u32, avg_price: f64}>}` with `active_portfolio` index
- `news_sources`: Vec of RSS feed URLs (defaults: CNBC Indonesia Market/News, IDX Channel, Tempo Bisnis)

**Config Migration**: Old configs with a flat `portfolio: Vec<Holding>` field are automatically migrated to the new `portfolios` format on first load. The old field uses `#[serde(default, skip_serializing)]` so it's read during migration but never written back.

## Code Conventions

- Remove unused functions/fields rather than keeping them for "future use"
- Input character filtering is done in `main.rs` key handlers (e.g., digits-only for numeric fields)
- Footer bar in `ui.rs::draw_footer()` provides context-sensitive prompts per `InputMode`
