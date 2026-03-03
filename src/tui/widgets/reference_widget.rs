use crate::reference::{Locale, SearchHit};
use crate::tui::{i18n::text, theme::Theme};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct ReferenceWidgetState<'a> {
    pub query: &'a str,
    pub mode: &'a str,
    pub locale: Locale,
    pub visible_count: usize,
    pub indexed_only_count: usize,
    pub hits: &'a [SearchHit],
    pub selected: usize,
}

pub fn render(frame: &mut Frame, area: Rect, state: ReferenceWidgetState<'_>, theme: &Theme) {
    let title = if state.query.is_empty() {
        text(state.locale, " ◉ INDEX TECHNIQUE ", " ◉ REFERENCE INDEX ")
    } else {
        text(
            state.locale,
            " ◉ RECHERCHE TECHNIQUE ",
            " ◉ REFERENCE SEARCH ",
        )
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

    if state.hits.is_empty() {
        let empty = if state.query.is_empty() {
            text(
                state.locale,
                "Appuyez sur / pour rechercher un terme technique.",
                "Press / to search technical terms.",
            )
        } else {
            text(
                state.locale,
                "Aucune entree ne correspond a cette recherche.",
                "No reference entry matches this query.",
            )
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
        Span::styled(
            format!("{}: ", text(state.locale, "requete", "query")),
            theme.highlight_style(),
        ),
        Span::raw(if state.query.is_empty() {
            text(state.locale, "(index)", "(index)")
        } else {
            state.query
        }),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", text(state.locale, "mode", "mode")),
            theme.highlight_style(),
        ),
        Span::raw(state.mode),
        Span::raw("  "),
        Span::styled(
            format!("{}: ", text(state.locale, "ui", "ui")),
            theme.highlight_style(),
        ),
        Span::raw(format!(
            "{} {} / {} {}",
            state.visible_count,
            text(state.locale, "visibles", "visible"),
            state.indexed_only_count,
            text(state.locale, "indexes", "indexed"),
        )),
    ]));
    lines.push(Line::default());

    for (index, hit) in state
        .hits
        .iter()
        .take(inner.height.saturating_sub(3) as usize)
        .enumerate()
    {
        let marker = if index == state.selected { ">" } else { " " };
        let style = if index == state.selected {
            theme.highlight_style()
        } else {
            theme.muted_style()
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{marker} {} ", hit.entry.panel), style),
            Span::styled(hit.entry.title, style),
            Span::raw(" "),
            Span::styled(hit.entry.category, theme.highlight_style()),
            Span::raw(" "),
            Span::styled(
                format!("{:?}/{:?}", hit.entry.status, hit.entry.ui_visibility),
                theme.muted_style(),
            ),
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
