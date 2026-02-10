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
│   ├── portfolio.rs # Portfolio CRUD: add wizard, remove, detail, chart allocation
│   ├── export.rs    # Export menu logic, CSV/JSON formatters
│   ├── filter.rs    # Search/filter, get_filtered_watchlist/portfolio, selected_*_symbol
│   └── sort.rs      # Column comparison functions for watchlist and portfolio sorting
├── ui/
│   ├── mod.rs       # draw() entry point, draw_header(), draw_footer()
│   ├── tables.rs    # Column defs, visible_columns(), draw_watchlist(), draw_portfolio()
│   ├── modals.rs    # draw_help(), draw_export_menu(), draw_portfolio_chart()
│   ├── detail.rs    # draw_stock_detail() with section helpers (price, range, fundamentals, risk)
│   └── formatters.rs # format_price, format_change, format_compact, format_pl, etc.
├── config.rs        # Persistent config: watchlists, holdings, JSON file storage
└── api/
    ├── mod.rs       # Re-exports
    └── yahoo.rs     # Yahoo Finance API client with crumb authentication
```

### Key Patterns

**State Machine**: `InputMode` enum controls UI behavior (Normal, Adding, WatchlistAdd, WatchlistRename, StockDetail, PortfolioAddSymbol/Lots/Price, Help, Search, ExportMenu). `ViewMode` toggles between Watchlist and Portfolio views.

**Data Flow**: `App` holds `Config` (persistent), `quotes` cache (volatile), and `YahooClient`. Quote refresh is triggered by timer or user action. Config saves to `~/.config/idx-cli/config.json`.

**Yahoo Finance Auth**: The `YahooClient` fetches a crumb token from the main page and uses cookies for authenticated API requests. Crumb refresh happens on 401 responses.

**Multi-Step Input**: Portfolio add uses a wizard flow (Symbol → Lots → Price) with `pending_symbol`/`pending_lots` fields on `App`. Each step validates independently. `cancel_portfolio_add()` resets all pending state. Prefer this pattern over comma-separated single-line input for multi-field forms.

**IDX Symbol Convention**: Stock codes (e.g., "BBCA") are internally converted to Yahoo format ("BBCA.JK") for API calls, then stripped back for display.

**Responsive Columns**: `ColumnDef` structs with priority tiers (1=always visible, 4=only on wide terminals). `visible_columns()` greedily includes columns from highest to lowest priority until available width is exhausted. Stretch columns (Name for watchlist, Value for portfolio) absorb extra space via `Constraint::Min()`.

**Column Sorting**: `SortDirection` enum with `cycle_sort_column()` / `toggle_sort_direction()`. Sort applied in `get_filtered_watchlist()` / `get_filtered_portfolio()` after search filtering. Free functions `compare_watchlist_column()` / `compare_portfolio_column()` handle per-column type-aware comparison. Export functions use `get_sorted_watchlist()` (insertion order, not sorted).

**Selection via Filtered View**: `selected_index` / `portfolio_selected` are indices into the **filtered/sorted** list, not the raw config list. All operations (delete, detail view, navigation bounds) must resolve through `selected_watchlist_symbol()` / `selected_portfolio_symbol()` which index the filtered list returned by `get_filtered_watchlist()` / `get_filtered_portfolio()`.

## Keybindings (Normal Mode)

| Key | Watchlist | Portfolio |
|-----|-----------|-----------|
| `p` | Switch to Portfolio | Switch to Watchlist |
| `a` | Add stock | Add holding (step-by-step) |
| `d` | Delete selected | Delete selected |
| `r` | Refresh quotes | Refresh quotes |
| `Enter` | Stock detail popup | Stock detail popup |
| `j/k` or `↑/↓` | Navigate | Navigate |
| `h/l` or `←/→` | Prev/Next watchlist | - |
| `n` | New watchlist | - |
| `R` | Rename watchlist | - |
| `D` | Delete watchlist | - |
| `s` | Cycle sort column | Cycle sort column |
| `S` | Toggle sort direction | Toggle sort direction |
| `c` | - | Portfolio allocation chart |
| `q` | Quit | Quit |

## Config File

Stored at `~/.config/idx-cli/config.json`. Schema managed by `Config` struct in `config.rs`. Contains:
- `watchlists`: Vec of `{name, symbols: Vec<String>}` with `current_watchlist` index
- `portfolio`: Vec of `{symbol, lots: u32, avg_price: f64}`

## Code Conventions

- Remove unused functions/fields rather than keeping them for "future use"
- Input character filtering is done in `main.rs` key handlers (e.g., digits-only for numeric fields)
- Footer bar in `ui.rs::draw_footer()` provides context-sensitive prompts per `InputMode`
