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
        " {}: {}   {}: {}   {}: {}",
        text(state.locale, "Cache", "Cached"),
        display_mb(metrics.cached_kb, metrics.cached_supported),
        text(state.locale, "Buffers", "Buffers"),
        display_mb(metrics.buffers_kb, metrics.buffers_supported),
        text(state.locale, "Dirty", "Dirty"),
        display_mb(metrics.dirty_kb, metrics.dirty_supported),
    );
    frame.render_widget(
        Paragraph::new(cache_text).style(ratatui::style::Style::default().fg(theme.neutral)),
        chunks[2],
    );

    let vm_text = format!(
        " Pgflt: {}   Maj: {}   Scan: {}   {}: {}",
        display_u64(metrics.vm_pgfault, metrics.vm_fault_counters_supported),
        display_u64(metrics.vm_pgmajfault, metrics.vm_fault_counters_supported),
        display_u64(metrics.vm_pgscan, metrics.vm_scan_counters_supported),
        text(state.locale, "Reclaim", "Steal"),
        display_u64(metrics.vm_pgsteal, metrics.vm_scan_counters_supported),
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
        display_u64(metrics.vm_pgpgin, metrics.vm_io_counters_supported),
        display_u64(metrics.vm_pgpgout, metrics.vm_io_counters_supported),
        display_u64(metrics.vm_pswpin, metrics.vm_io_counters_supported),
        display_u64(metrics.vm_pswpout, metrics.vm_io_counters_supported),
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

fn display_mb(value_kb: u64, supported: bool) -> String {
    if supported {
        format!("{:.0} MB", value_kb as f64 / 1024.0)
    } else {
        "n/a".to_string()
    }
}

fn display_u64(value: u64, supported: bool) -> String {
    if supported {
        value.to_string()
    } else {
        "n/a".to_string()
    }
}
