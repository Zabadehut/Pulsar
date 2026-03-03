use crate::collectors::linux::{LinuxMetrics, PressureMetric};
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    metrics: Option<&LinuxMetrics>,
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " ◉ LINUX ",
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

    let Some(metrics) = metrics else {
        frame.render_widget(Paragraph::new("No Linux-specific metrics"), inner);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let cgroup_line = if let Some(cgroup) = metrics.cgroup.as_ref() {
        let memory_limit = cgroup
            .memory_max_bytes
            .map(format_gib)
            .unwrap_or_else(|| "unbounded".to_string());
        format!(
            " cgroup v{}  mem {} / {} ({:.0}%)  pids {}",
            cgroup.version,
            format_gib(cgroup.memory_current_bytes),
            memory_limit,
            cgroup.memory_usage_pct,
            cgroup.pids_current
        )
    } else {
        " cgroup v2 not detected".to_string()
    };
    frame.render_widget(paragraph(cgroup_line, theme), chunks[0]);

    let throttling_line = if let Some(cgroup) = metrics.cgroup.as_ref() {
        let throttling_pct = if cgroup.cpu_nr_periods > 0 {
            cgroup.cpu_nr_throttled as f64 / cgroup.cpu_nr_periods as f64 * 100.0
        } else {
            0.0
        };
        format!(
            " cpu throttle {:.1}%  usage {} ms  throttled {} ms",
            throttling_pct,
            cgroup.cpu_usage_usec / 1_000,
            cgroup.cpu_throttled_usec / 1_000
        )
    } else {
        " cpu throttle unavailable".to_string()
    };
    frame.render_widget(paragraph(throttling_line, theme), chunks[1]);

    let psi_cpu_mem = if let Some(psi) = metrics.psi.as_ref() {
        format!(
            " psi some avg10  cpu {}  mem {}",
            format_avg10(&psi.cpu),
            format_avg10(&psi.memory)
        )
    } else {
        " psi unavailable".to_string()
    };
    frame.render_widget(paragraph(psi_cpu_mem, theme), chunks[2]);

    let psi_io = if let Some(psi) = metrics.psi.as_ref() {
        format!(
            " psi io avg10 {}  path {}",
            format_avg10(&psi.io),
            cgroup_path(metrics)
        )
    } else {
        " psi io unavailable".to_string()
    };
    frame.render_widget(paragraph(psi_io, theme), chunks[3]);
}

fn paragraph(text: String, theme: &Theme) -> Paragraph<'static> {
    Paragraph::new(text).style(ratatui::style::Style::default().fg(theme.neutral))
}

fn format_avg10(metric: &PressureMetric) -> String {
    metric
        .some
        .as_ref()
        .map(|window| format!("{:.1}%", window.avg10))
        .unwrap_or_else(|| "n/a".to_string())
}

fn cgroup_path(metrics: &LinuxMetrics) -> String {
    metrics
        .cgroup
        .as_ref()
        .map(|cgroup| truncate_left(&cgroup.path, 24))
        .unwrap_or_else(|| "-".to_string())
}

fn truncate_left(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        value.to_string()
    } else {
        let suffix: String = value
            .chars()
            .skip(count.saturating_sub(max_chars.saturating_sub(1)))
            .collect();
        format!("…{}", suffix)
    }
}

fn format_gib(bytes: u64) -> String {
    format!("{:.1} GiB", bytes as f64 / 1_073_741_824.0)
}
