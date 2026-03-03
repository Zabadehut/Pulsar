use crate::collectors::MemoryMetrics;
use crate::reference::Locale;
use crate::tui::{i18n::text, theme::Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub struct MemoryWidgetState<'a> {
    pub metrics: Option<&'a MemoryMetrics>,
    pub memory_pressure: f64,
    pub locale: Locale,
    pub detailed: bool,
    pub highlighted: bool,
}

pub fn render(frame: &mut Frame, area: Rect, state: MemoryWidgetState<'_>, theme: &Theme) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            text(state.locale, " ◉ MEMOIRE ", " ◉ MEMORY "),
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
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ];
    if state.detailed && inner.height >= 8 {
        constraints.push(Constraint::Length(1));
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let ram_pct = metrics.usage_pct.clamp(0.0, 100.0);
    frame.render_widget(
        Gauge::default()
            .gauge_style(theme.gauge_for_pct(ram_pct))
            .ratio(ram_pct / 100.0)
            .label(format!(
                "{}  {:.1} / {:.1} GB  ({:.1}%)",
                text(state.locale, "RAM", "RAM"),
                metrics.used_kb as f64 / 1_048_576.0,
                metrics.total_kb as f64 / 1_048_576.0,
                ram_pct,
            )),
        chunks[0],
    );

    let swap_pct = if metrics.swap_total_kb > 0 {
        (metrics.swap_used_kb as f64 / metrics.swap_total_kb as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    let swap_label = if metrics.swap_total_kb > 0 {
        format!(
            "{} {:.1} / {:.1} GB  ({:.1}%)",
            text(state.locale, "Swap", "Swap"),
            metrics.swap_used_kb as f64 / 1_048_576.0,
            metrics.swap_total_kb as f64 / 1_048_576.0,
            swap_pct,
        )
    } else {
        text(state.locale, "Swap  —  desactive", "Swap  —  disabled").to_string()
    };
    frame.render_widget(
        Gauge::default()
            .gauge_style(theme.gauge_for_pct(swap_pct))
            .ratio(swap_pct / 100.0)
            .label(swap_label),
        chunks[1],
    );

    let cache_text = format!(
        " {}: {:.0} MB   {}: {:.0} MB   {}: {:.0} MB",
        text(state.locale, "Cache", "Cached"),
        metrics.cached_kb as f64 / 1024.0,
        text(state.locale, "Buffers", "Buffers"),
        metrics.buffers_kb as f64 / 1024.0,
        text(state.locale, "Dirty", "Dirty"),
        metrics.dirty_kb as f64 / 1024.0,
    );
    frame.render_widget(
        Paragraph::new(cache_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[2],
    );

    let vm_text = format!(
        " Pgflt: {}   Maj: {}   Scan: {}   {}: {}",
        metrics.vm_pgfault,
        metrics.vm_pgmajfault,
        metrics.vm_pgscan,
        text(state.locale, "Reclaim", "Steal"),
        metrics.vm_pgsteal,
    );
    frame.render_widget(
        Paragraph::new(vm_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[3],
    );

    let pressure_text = format!(
        " {}: {:.0} MB   {}: {:.0}%   PgIn/Out: {}/{}   SwpIn/Out: {}/{}",
        text(state.locale, "Disponible", "Available"),
        metrics.available_kb as f64 / 1024.0,
        text(state.locale, "Pression", "Pressure"),
        state.memory_pressure * 100.0,
        metrics.vm_pgpgin,
        metrics.vm_pgpgout,
        metrics.vm_pswpin,
        metrics.vm_pswpout,
    );
    frame.render_widget(
        Paragraph::new(pressure_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[4],
    );

    if state.detailed && chunks.len() > 5 {
        let extra_text = format!(
            " {}: {:.0} MB   {}: {:.0} MB",
            text(state.locale, "Utilisee", "Used"),
            metrics.used_kb as f64 / 1024.0,
            text(state.locale, "Libre", "Free"),
            metrics.free_kb as f64 / 1024.0,
        );
        frame.render_widget(
            Paragraph::new(extra_text).style(ratatui::style::Style::default().fg(theme.neutral)),
            chunks[5],
        );
    }
}
