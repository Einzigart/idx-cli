# idx-cli

A terminal UI (TUI) application for tracking Indonesian stock market (IDX) data.

![Rust](https://img.shields.io/badge/rust-2024-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **Real-time stock quotes** from Yahoo Finance
- **Multiple watchlists** - organize stocks by category
- **Portfolio tracking** - track holdings with P/L calculations
- **Stock detail popup** - view extended quote information

## Installation

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/idx-cli.git
cd idx-cli

# Build and run
cargo build --release
./target/release/idx-cli
```

## Usage

```bash
# Run with default 5-second refresh interval
idx-cli

# Run with custom refresh interval (in seconds)
idx-cli -i 10
```

## Keybindings

| Key | Watchlist View | Portfolio View |
|-----|----------------|----------------|
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

## Configuration

Configuration is stored at `~/.config/idx-cli/config.json` and includes:
- Watchlists with stock symbols
- Portfolio holdings (symbol, lots, average price)

## License

MIT
