use crate::reference::SearchHit;
use crate::tui::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    hits: &[SearchHit],
    selected: usize,
    theme: &Theme,
) {
    let title = if query.is_empty() {
        " ◉ REFERENCE INDEX "
    } else {
        " ◉ REFERENCE SEARCH "
    };

    let block = Block::default()
        .title(Line::from(vec![Span::styled(title, theme.title_style())]))
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    if hits.is_empty() {
        let empty = if query.is_empty() {
            "Press / to search technical terms."
        } else {
            "No reference entry matches this query."
        };
        frame.render_widget(
            Paragraph::new(empty)
                .style(theme.muted_style())
                .wrap(Wrap { trim: true }),
            inner,
        );
        return;
    }

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("query: ", theme.highlight_style()),
        Span::raw(if query.is_empty() { "(index)" } else { query }),
    ]));
    lines.push(Line::default());

    for (index, hit) in hits
        .iter()
        .take(inner.height.saturating_sub(2) as usize)
        .enumerate()
    {
        let marker = if index == selected { ">" } else { " " };
        let style = if index == selected {
            theme.highlight_style()
        } else {
            theme.muted_style()
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{marker} {} ", hit.entry.panel), style),
            Span::styled(hit.entry.title, style),
        ]));
        lines.push(Line::from(hit.entry.summary));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .style(theme.muted_style())
            .wrap(Wrap { trim: true }),
        inner,
    );
}
