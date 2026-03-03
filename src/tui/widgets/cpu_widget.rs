use crate::collectors::CpuMetrics;
use crate::reference::Locale;
use crate::tui::{i18n::text, theme::Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub struct CpuWidgetState<'a> {
    pub metrics: Option<&'a CpuMetrics>,
    pub trend_p50: f64,
    pub trend_p95: f64,
    pub locale: Locale,
    pub detailed: bool,
    pub highlighted: bool,
}

pub fn render(frame: &mut Frame, area: Rect, state: CpuWidgetState<'_>, theme: &Theme) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            text(state.locale, " ◉ CPU ", " ◉ CPU "),
            if state.highlighted {
                theme.highlight_style()
            } else {
                theme.title_style()
            },
        )]))
        .borders(Borders::ALL)
        .border_style(if state.highlighted {
            theme.highlight_style()
        } else {
            theme.border_style()
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(metrics) = state.metrics else {
        frame.render_widget(
            Paragraph::new(text(state.locale, "Collecte...", "Collecting...")),
            inner,
        );
        return;
    };

    let mut constraints = vec![
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ];
    if state.detailed && inner.height >= 7 {
        constraints.push(Constraint::Length(1));
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let pct = metrics.global_usage_pct.clamp(0.0, 100.0);
    frame.render_widget(
        Gauge::default()
            .gauge_style(theme.gauge_for_pct(pct))
            .ratio(pct / 100.0)
            .label(format!("{pct:.1}%")),
        chunks[0],
    );

    let load_text = format!(
        " {}: {:.2}  {:.2}  {:.2}  (1m  5m  15m)",
        text(state.locale, "Charge", "Load"),
        metrics.load_avg_1,
        metrics.load_avg_5,
        metrics.load_avg_15
    );
    frame.render_widget(
        Paragraph::new(load_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[1],
    );

    let mode_text = format!(
        " usr: {:.1}%  nice: {:.1}%  {}: {:.1}%  idle: {:.1}%",
        metrics.modes.user_pct,
        metrics.modes.nice_pct,
        text(state.locale, "sys", "sys"),
        metrics.modes.system_pct,
        metrics.modes.idle_pct,
    );
    frame.render_widget(
        Paragraph::new(mode_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[2],
    );

    let irq_text = format!(
        " iow: {:.1}%  irq: {:.1}%  sirq: {:.1}%  {}: {:.1}%",
        metrics.modes.iowait_pct,
        metrics.modes.irq_pct,
        metrics.modes.softirq_pct,
        text(state.locale, "vol", "stl"),
        metrics.modes.steal_pct,
    );
    frame.render_widget(
        Paragraph::new(irq_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[3],
    );

    let trend_text = format!(
        " ctx: {}  irq: {}  p50/p95: {:.1}/{:.1}",
        metrics.context_switches, metrics.interrupts, state.trend_p50, state.trend_p95
    );
    frame.render_widget(
        Paragraph::new(trend_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[4],
    );

    if state.detailed && chunks.len() > 5 {
        let hottest = metrics
            .per_core
            .iter()
            .take(8)
            .map(|core| format!("c{}:{:.0}%", core.id, core.usage_pct))
            .collect::<Vec<_>>()
            .join("  ");
        frame.render_widget(
            Paragraph::new(format!(
                " {} {}",
                text(state.locale, "coeurs:", "cores:"),
                hottest
            ))
            .style(ratatui::style::Style::default().fg(theme.neutral)),
            chunks[5],
        );
    }
}
