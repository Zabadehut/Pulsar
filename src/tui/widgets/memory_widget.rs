use crate::collectors::MemoryMetrics;
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
    metrics: Option<&MemoryMetrics>,
    memory_pressure: f64,
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " ◉ MEMORY ",
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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // RAM gauge
            Constraint::Length(2), // Swap gauge
            Constraint::Length(1), // Details
            Constraint::Length(1), // VM counters
            Constraint::Length(1), // Available / pressure / paging
        ])
        .split(inner);

    // RAM
    let pct = m.usage_pct.clamp(0.0, 100.0);
    let label = format!(
        "RAM  {:.1} / {:.1} GB  ({:.1}%)",
        m.used_kb as f64 / 1_048_576.0,
        m.total_kb as f64 / 1_048_576.0,
        pct,
    );
    frame.render_widget(
        Gauge::default()
            .gauge_style(theme.gauge_for_pct(pct))
            .ratio(pct / 100.0)
            .label(label),
        chunks[0],
    );

    // Swap
    let swap_pct = if m.swap_total_kb > 0 {
        (m.swap_used_kb as f64 / m.swap_total_kb as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    let swap_label = if m.swap_total_kb > 0 {
        format!(
            "Swap {:.1} / {:.1} GB  ({:.1}%)",
            m.swap_used_kb as f64 / 1_048_576.0,
            m.swap_total_kb as f64 / 1_048_576.0,
            swap_pct,
        )
    } else {
        "Swap  —  disabled".to_string()
    };
    frame.render_widget(
        Gauge::default()
            .gauge_style(theme.gauge_for_pct(swap_pct))
            .ratio(swap_pct / 100.0)
            .label(swap_label),
        chunks[1],
    );

    // Cache / Buffers / Dirty
    let detail = format!(
        " Cached: {:.0} MB   Buffers: {:.0} MB   Dirty: {:.0} MB",
        m.cached_kb as f64 / 1024.0,
        m.buffers_kb as f64 / 1024.0,
        m.dirty_kb as f64 / 1024.0,
    );
    frame.render_widget(
        Paragraph::new(detail).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[2],
    );

    let vm_detail = format!(
        " Pgflt: {}   Maj: {}   Scan: {}   Steal: {}",
        m.vm_pgfault, m.vm_pgmajfault, m.vm_pgscan, m.vm_pgsteal,
    );
    frame.render_widget(
        Paragraph::new(vm_detail).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[3],
    );

    let extra = format!(
        " Available: {:.0} MB   Pressure: {:.0}%   PgIn/Out: {}/{}   SwpIn/Out: {}/{}",
        m.available_kb as f64 / 1024.0,
        memory_pressure * 100.0,
        m.vm_pgpgin,
        m.vm_pgpgout,
        m.vm_pswpin,
        m.vm_pswpout,
    );
    frame.render_widget(
        Paragraph::new(extra).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[4],
    );
}
