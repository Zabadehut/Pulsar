use crate::collectors::CpuMetrics;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    metrics: Option<&CpuMetrics>,
    trend_p50: f64,
    trend_p95: f64,
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " ◉ CPU ",
            if highlighted {
                theme.highlight_style()
            } else {
                theme.title_style()
            },
        )]))
        .borders(Borders::ALL)
        .border_style(if highlighted {
            theme.highlight_style()
        } else {
            theme.border_style()
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(m) = metrics else {
        frame.render_widget(Paragraph::new("Collecting..."), inner);
        return;
    };

    // Layout : gauge global + load avg + stats
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // gauge global
            Constraint::Length(1), // load avg
            Constraint::Length(1), // user / nice / system / idle
            Constraint::Length(1), // iowait / irq / softirq / steal
            Constraint::Length(1), // context switches / interrupts / trend
        ])
        .split(inner);

    // Gauge CPU global
    let pct = m.global_usage_pct.clamp(0.0, 100.0);
    let label = format!("{:.1}%", pct);
    let gauge = Gauge::default()
        .gauge_style(theme.gauge_for_pct(pct))
        .ratio(pct / 100.0)
        .label(label);
    frame.render_widget(gauge, chunks[0]);

    // Load averages
    let load_text = format!(
        " Load: {:.2}  {:.2}  {:.2}  (1m  5m  15m)",
        m.load_avg_1, m.load_avg_5, m.load_avg_15
    );
    frame.render_widget(
        Paragraph::new(load_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[1],
    );

    let mode_text = format!(
        " usr: {:.1}%  nice: {:.1}%  sys: {:.1}%  idle: {:.1}%",
        m.modes.user_pct, m.modes.nice_pct, m.modes.system_pct, m.modes.idle_pct,
    );
    frame.render_widget(
        Paragraph::new(mode_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[2],
    );

    let irq_text = format!(
        " iow: {:.1}%  irq: {:.1}%  sirq: {:.1}%  stl: {:.1}%",
        m.modes.iowait_pct, m.modes.irq_pct, m.modes.softirq_pct, m.modes.steal_pct,
    );
    frame.render_widget(
        Paragraph::new(irq_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[3],
    );

    let detail_text = format!(
        " ctx: {}  irq: {}  p50/p95: {:.1}/{:.1}",
        m.context_switches, m.interrupts, trend_p50, trend_p95
    );
    frame.render_widget(
        Paragraph::new(detail_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[4],
    );
}
