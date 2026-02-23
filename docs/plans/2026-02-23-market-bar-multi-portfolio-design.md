# Market Summary Bar & Multiple Portfolios Design

## Feature 1: Market Summary Bar (IHSG in Header)

### Overview
Display IHSG (Jakarta Composite Index) data right-aligned in the existing header row. No new layout rows — the header absorbs the market data alongside the existing title, view indicator, and clock.

### Data Source
- The symbol `^JKSE` is appended to every `get_quotes()` call alongside user stocks.
- `YahooClient::to_yahoo_symbol()` must pass through `^`-prefixed index symbols unchanged (no `.JK` suffix).
- The IHSG quote lives in the existing `quotes: HashMap` — no new state fields.

### Display Format
```
 IDX Stock Tracker | Banking (1/3)              IHSG 7,234 ▲+1.23% [14:30:05]
```
- Green text for positive change, red for negative, gray for zero.
- When quote is unavailable: `IHSG ---`.
- The clock moves to the far right after IHSG data.

### Code Changes
- `api/yahoo.rs`: `to_yahoo_symbol()` skips `.JK` for `^`-prefixed symbols. `From<QuoteResult>` maps `^JKSE` display symbol to `IHSG`.
- `app/mod.rs`: `refresh_symbols()` always appends `"^JKSE"`. Add `get_ihsg_quote() -> Option<&StockQuote>`.
- `ui/mod.rs`: `draw_header()` renders IHSG spans right-aligned using `Line::from()` with right-aligned layout.

---

## Feature 2: Multiple Portfolios

### Overview
Transform the single flat portfolio into named portfolio groups, mirroring the existing watchlist pattern exactly.

### Config Changes (`config.rs`)

New struct:
```rust
struct Portfolio {
    name: String,
    holdings: Vec<Holding>,
}
```

Config field changes:
- `portfolio: Vec<Holding>` → `portfolios: Vec<Portfolio>`
- Add `active_portfolio: usize` (default 0)

### Migration
In `Config::load()`, detect old format (flat `portfolio` array of Holdings) and migrate to `portfolios: [Portfolio { name: "Default", holdings: <old> }]`. Save immediately after migration. Use serde's `default` + custom deserialization or a migration function like `migrate_news_sources()`.

### Config CRUD Methods
Mirror watchlist methods:
- `current_portfolio()` / `current_portfolio_mut()` → `&Portfolio` / `&mut Portfolio`
- `next_portfolio()` / `prev_portfolio()` — wrapping index
- `add_portfolio(name)` — push + set active
- `remove_portfolio()` — remove if >1, clamp index
- `rename_portfolio(name)` — rename current
- `portfolio_symbols()` — delegates to `current_portfolio().holdings`

### App State (`app/mod.rs`)
New `InputMode` variants:
- `PortfolioNew` — entering new portfolio name
- `PortfolioRename` — renaming current portfolio

### Navigation & Keybindings (`main.rs`)
In Portfolio view:
- `←/→` or `h/l` — switch between portfolios
- `n` — create new portfolio (enters `PortfolioNew` mode)
- `R` — rename current portfolio (enters `PortfolioRename` mode)
- `D` — delete current portfolio (if >1)

### Header Indicator (`ui/mod.rs`)
Portfolio view header shows: `PortfolioName (n/N)` — same format as watchlist indicator.

### Footer Hints (`ui/mod.rs`)
Portfolio footer adds `[←→] Port` navigation hint.

### Files Changed
| File | Change |
|------|--------|
| `config.rs` | `Portfolio` struct, migration, CRUD methods |
| `app/mod.rs` | `PortfolioNew`/`PortfolioRename` InputMode variants, `portfolio_indicator()` |
| `app/portfolio.rs` | All methods use `current_portfolio()` instead of `config.portfolio` |
| `app/filter.rs` | `get_filtered_portfolio()` reads from current portfolio |
| `app/export.rs` | Export reads from current portfolio |
| `ui/mod.rs` | Header indicator, footer hints |
| `main.rs` | Key handlers for portfolio nav/CRUD |

### Testing
- Config migration: old format → new format preserves data
- Portfolio CRUD: add/remove/rename with index clamping
- Portfolio switching: next/prev wrapping behavior
