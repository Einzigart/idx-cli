# idx-cli

A terminal UI (TUI) application for tracking Indonesian stock market (IDX) data.

![Rust](https://img.shields.io/badge/rust-2024-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

![idx-cli screenshot](assets/home.png)

## Features

- **Real-time stock quotes** from Yahoo Finance
- **Multiple watchlists** — organize stocks by category
- **Multiple portfolios** — track holdings with P/L calculations and allocation chart
- **RSS news feed** — aggregated financial headlines from Indonesian sources
- **Bookmark articles** — save news articles for later with read/unread tracking
- **Price alerts** — set target price or percentage alerts with desktop notifications
- **Stock detail popup** — price, fundamentals, risk metrics, 3-month sparkline chart, and related news
- **Export** — save watchlist or portfolio data as CSV or JSON
- **Search & sort** — filter by symbol/headline and sort by any column

## Installation

```bash
# Clone the repository
git clone https://github.com/Einzigart/idx-cli.git
cd idx-cli

# Build and run
cargo build --release
./target/release/idx-cli
```

## Usage

```bash
# Run with default 1-second refresh interval
idx-cli

# Run with custom refresh interval (in seconds)
idx-cli -i 10
```

## Keybindings

### General

| Key | Action |
|-----|--------|
| `p` | Cycle view: Watchlist → Portfolio → News |
| `j/k` or `↑/↓` | Navigate list |
| `s` | Cycle sort column |
| `S` | Toggle sort direction |
| `/` | Search / filter |
| `?` | Help |
| `q` | Quit |

### Watchlist

| Key | Action |
|-----|--------|
| `a` | Add stock symbol |
| `d` | Delete selected stock |
| `e` | Export data (CSV/JSON) |
| `r` | Refresh quotes |
| `A` | Manage price alerts |
| `Enter` | Stock detail popup |
| `h/l` or `←/→` | Previous / next watchlist |
| `n` | New watchlist |
| `R` | Rename watchlist |
| `D` | Delete watchlist |

### Portfolio

| Key | Action |
|-----|--------|
| `a` | Add holding (step-by-step) |
| `e` | Edit selected holding |
| `d` | Delete selected holding |
| `r` | Refresh quotes |
| `A` | Manage price alerts |
| `c` | Portfolio allocation chart |
| `Enter` | Stock detail popup |
| `h/l` or `←/→` | Previous / next portfolio |
| `n` | New portfolio |
| `R` | Rename portfolio |
| `D` | Delete portfolio |

### News — Feed tab

| Key | Action |
|-----|--------|
| `h/l` or `←/→` | Switch to Bookmarks tab |
| `b` | Toggle bookmark on article |
| `r` | Refresh news feeds |
| `Enter` | Open article preview |

In article preview: `b` bookmark, `o` open in browser, `↑/↓` scroll, `Esc` close.

### News — Bookmarks tab

| Key | Action |
|-----|--------|
| `h/l` or `←/→` | Switch to Feed tab |
| `Enter` | Open bookmark detail (marks as read) |
| `d` | Remove selected bookmark |
| `D` | Clear all bookmarks |
| `m` | Toggle read / unread |

In bookmark detail: `o` open in browser, `m` toggle read, `↑/↓` scroll, `Esc` close.

### Price Alerts (`A` from Watchlist or Portfolio)

| Key | Action |
|-----|--------|
| `Enter` | Toggle enabled/disabled, or add new alert |
| `d` | Delete selected alert |
| `↑/↓` | Navigate |
| `Esc` | Close |

Adding an alert is a two-step wizard: select type (Above / Below / % Gain / % Loss), then enter target value.

## Configuration

Configuration is stored at `~/.config/idx-cli/config.json` and includes:
- Watchlists with stock symbols
- Portfolio holdings (symbol, lots, average price)
- RSS news source URLs
- Price alerts (type, target value, cooldown)
- Bookmarked articles with read/unread state

## License

MIT
