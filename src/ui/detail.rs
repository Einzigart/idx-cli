use crate::api::{NewsItem, StockQuote};
use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Sparkline},
    Frame,
};
use super::centered_rect;
use super::formatters::*;

fn section_divider<'a>(title: &str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("─── {} ", title), Style::default().fg(Color::Yellow)),
        Span::styled("───────────────────────────", Style::default().fg(Color::DarkGray)),
    ])
}

fn detail_header(q: &StockQuote) -> Vec<Line<'static>> {
    let sector = q.sector.as_deref().unwrap_or("N/A");
    let industry = q.industry.as_deref().unwrap_or("N/A");
    vec![
        Line::from(Span::styled(
            q.long_name.as_deref().unwrap_or(&q.short_name).to_string(),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("{} | {}", sector, industry),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ]
}

fn detail_price_section(q: &StockQuote) -> Vec<Line<'static>> {
    let change_color = if q.change >= 0.0 { Color::Green } else { Color::Red };
    let gap_percent = if q.prev_close > 0.0 {
        ((q.open - q.prev_close) / q.prev_close) * 100.0
    } else {
        0.0
    };
    let gap_color = if gap_percent >= 0.0 { Color::Green } else { Color::Red };

    vec![
        section_divider("Price"),
        Line::from(vec![
            Span::raw("Current:        "),
            Span::styled(format_price(q.price), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Change:         "),
            Span::styled(
                format!("{} ({:+.2}%)", format_change(q.change), q.change_percent),
                Style::default().fg(change_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Open:           "),
            Span::raw(format_price(q.open)),
            Span::raw("  Prev Close: "),
            Span::raw(format_price(q.prev_close)),
        ]),
        Line::from(vec![
            Span::raw("Gap:            "),
            Span::styled(format!("{:+.2}%", gap_percent), Style::default().fg(gap_color)),
        ]),
        Line::from(""),
    ]
}

fn detail_range_section(q: &StockQuote) -> Vec<Line<'static>> {
    let day_range = q.high - q.low;
    let day_range_percent = if day_range > 0.0 {
        ((q.price - q.low) / day_range) * 100.0
    } else {
        50.0
    };

    let mut lines = vec![
        section_divider("Day Range"),
        Line::from(vec![
            Span::raw("High:           "),
            Span::styled(format_price(q.high), Style::default().fg(Color::Green)),
            Span::raw("  Low: "),
            Span::styled(format_price(q.low), Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::raw("Position:       "),
            Span::raw(format!("{:.1}% from low", day_range_percent)),
        ]),
        Line::from(""),
        section_divider("52-Week Range"),
    ];

    let w52_high = q.fifty_two_week_high.map(format_price).unwrap_or_else(|| "N/A".to_string());
    let w52_low = q.fifty_two_week_low.map(format_price).unwrap_or_else(|| "N/A".to_string());
    lines.push(Line::from(vec![
        Span::raw("52W High:       "),
        Span::styled(w52_high, Style::default().fg(Color::Green)),
        Span::raw("  52W Low: "),
        Span::styled(w52_low, Style::default().fg(Color::Red)),
    ]));
    if let (Some(high), Some(low)) = (q.fifty_two_week_high, q.fifty_two_week_low)
        && high > low
    {
        let pos = ((q.price - low) / (high - low)) * 100.0;
        lines.push(Line::from(vec![
            Span::raw("Position:       "),
            Span::raw(format!("{:.1}% from 52W low", pos)),
        ]));
    }
    lines
}

fn detail_fundamentals_section(q: &StockQuote) -> Vec<Line<'static>> {
    let market_cap_str = q.market_cap.map(format_market_cap).unwrap_or_else(|| "N/A".to_string());
    let pe_str = q.trailing_pe.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".to_string());
    let div_yield_str = q.dividend_yield
        .map(|v| format!("{:.2}%", v * 100.0))
        .unwrap_or_else(|| "N/A".to_string());

    vec![
        Line::from(""),
        section_divider("Fundamentals"),
        Line::from(vec![
            Span::raw("Market Cap:     "),
            Span::styled(market_cap_str, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("P/E Ratio:      "),
            Span::raw(pe_str),
            Span::raw("  Div Yield: "),
            Span::raw(div_yield_str),
        ]),
    ]
}

fn detail_risk_section(q: &StockQuote) -> Vec<Line<'static>> {
    let value = q.price * q.volume as f64;
    let beta_str = q.beta.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".to_string());
    let avg_vol_str = q.average_volume.map(format_volume).unwrap_or_else(|| "N/A".to_string());

    vec![
        Line::from(""),
        section_divider("Risk & Liquidity"),
        Line::from(vec![Span::raw("Beta:           "), Span::raw(beta_str)]),
        Line::from(vec![
            Span::raw("Volume:         "),
            Span::raw(format_volume(q.volume)),
            Span::raw("  Avg Vol: "),
            Span::raw(avg_vol_str),
        ]),
        Line::from(vec![
            Span::raw("Value:          "),
            Span::styled(format_value(value), Style::default().fg(Color::Cyan)),
        ]),
    ]
}

fn detail_news_section(news: Option<&[NewsItem]>, loading: bool) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(""), section_divider("News")];

    if loading {
        lines.push(Line::from(Span::styled("Loading news...", Style::default().fg(Color::DarkGray))));
        return lines;
    }

    let items = match news {
        Some(items) if !items.is_empty() => items,
        _ => {
            lines.push(Line::from(Span::styled("No news available", Style::default().fg(Color::DarkGray))));
            return lines;
        }
    };

    for item in items.iter().take(3) {
        lines.push(Line::from(Span::raw(item.title.clone())));
        let time = format_relative_time(item.published_at);
        let meta = if time.is_empty() {
            item.publisher.clone()
        } else {
            format!("{} - {}", item.publisher, time)
        };
        lines.push(Line::from(Span::styled(
            meta,
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines
}

pub fn draw_stock_detail(frame: &mut Frame, app: &App) {
    let area = centered_rect(55, 85, frame.area());
    frame.render_widget(Clear, area);

    let quote = match app.get_detail_quote() {
        Some(q) => q,
        None => return,
    };

    let title = format!(" {} - Stock Detail ", quote.symbol);
    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(20), Constraint::Length(5)])
        .split(inner_area);

    let mut content = detail_header(quote);
    content.extend(detail_price_section(quote));
    content.extend(detail_range_section(quote));
    content.extend(detail_fundamentals_section(quote));
    content.extend(detail_risk_section(quote));
    content.extend(detail_news_section(
        app.detail_news.as_deref(),
        app.news_loading,
    ));
    content.push(Line::from(""));
    content.push(Line::from(Span::styled("[Enter/Esc] Close", Style::default().fg(Color::DarkGray))));

    frame.render_widget(Paragraph::new(content).alignment(Alignment::Left), chunks[0]);
    draw_sparkline(frame, chunks[1], app);
}

fn draw_sparkline(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(ref chart) = app.detail_chart {
        let chart_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(10), Constraint::Min(20)])
            .split(area);

        let min = chart.low;
        let max = chart.high;
        let range = max - min;
        let data: Vec<u64> = if range > 0.0 {
            chart.closes.iter().map(|&v| ((v - min) / range * 100.0) as u64).collect()
        } else {
            chart.closes.iter().map(|_| 50u64).collect()
        };

        let y_axis_content = vec![
            Line::from(Span::styled(format_price(max), Style::default().fg(Color::Green))),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(format_price(min), Style::default().fg(Color::Red))),
        ];
        let y_axis = Paragraph::new(y_axis_content)
            .alignment(Alignment::Right)
            .block(Block::default().title(" 3M ").borders(Borders::TOP));
        frame.render_widget(y_axis, chart_chunks[0]);

        let sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::TOP))
            .data(&data)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(sparkline, chart_chunks[1]);
    } else if app.chart_loading {
        let loading = Paragraph::new("Loading chart...")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(loading, area);
    } else {
        let no_data = Paragraph::new("Chart data unavailable")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(no_data, area);
    }
}
