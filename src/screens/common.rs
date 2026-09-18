use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Table, Widget},
};
use ratatui_textarea::TextArea;

use crate::{
    app::{App, AppScreen},
    theme,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Status {
    Loading,
    Refreshing,
    Ready,
    Error,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Self::Loading => "loading",
            Self::Refreshing => "refreshing",
            Self::Ready => "ready",
            Self::Error => "error",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Loading | Self::Refreshing => theme::WARN,
            Self::Ready => theme::OK,
            Self::Error => theme::ERROR,
        }
    }

    fn is_busy(self) -> bool {
        matches!(self, Self::Loading | Self::Refreshing)
    }
}

pub fn current_status(app: &App) -> Status {
    let has_cached_targets = match app.warpgate_targets.as_ref() {
        Err(_) => return Status::Error,
        Ok(targets) => !targets.is_empty(),
    };

    match (app.loading_targets, has_cached_targets) {
        (true, true) => Status::Refreshing,
        (true, false) => Status::Loading,
        (false, _) => Status::Ready,
    }
}

/// Splits `area` so that `right` gets exactly the width it needs and the rest goes to the left
/// half, which keeps the two from overwriting each other in a narrow terminal.
pub fn split_row(right: &Line, area: Rect) -> (Rect, Rect) {
    let width = u16::try_from(right.width())
        .unwrap_or(u16::MAX)
        .min(area.width);

    let [left_area, right_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(width)]).areas(area);

    (left_area, right_area)
}

pub fn draw_split_row(left: Line, right: Line, area: Rect, buf: &mut Buffer) {
    let (left_area, right_area) = split_row(&right, area);

    Paragraph::new(left).render(left_area, buf);
    Paragraph::new(right).render(right_area, buf);
}

pub fn draw_rule(area: Rect, buf: &mut Buffer) {
    Block::new()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme::MUTED))
        .render(area, buf);
}

/// A key and what it does, as the footers and the empty-body hints all render them.
pub fn key_span(key: &str, label: &str, enabled: bool) -> [Span<'static>; 2] {
    [
        Span::styled(
            key.to_string(),
            Style::default().fg(if enabled { theme::KEY } else { theme::MUTED }),
        ),
        Span::styled(format!(" {label}"), Style::default().fg(theme::MUTED)),
    ]
}

/// Joins `hints` into one line, separated by `separator` spaces.
pub fn key_hints(hints: &[(&str, &str)], separator: usize) -> Line<'static> {
    let mut spans = Vec::new();
    for (key, label) in hints {
        if !spans.is_empty() {
            spans.push(Span::raw(" ".repeat(separator)));
        }
        spans.extend(key_span(key, label, true));
    }
    Line::from(spans)
}

/// The overlay shell every modal shares: clears `area`, draws the border and returns the inner
/// rect to draw into.
pub fn draw_modal(
    title: &str,
    width: u16,
    height: u16,
    padding: Padding,
    area: Rect,
    buf: &mut Buffer,
) -> Rect {
    let modal_area = centered_rect(width, height, area);
    Clear.render(modal_area, buf);
    draw_card(title, padding, modal_area, buf)
}

/// The bordered, titled block a modal or a settings card is drawn inside.
pub fn draw_card(title: &str, padding: Padding, area: Rect, buf: &mut Buffer) -> Rect {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::ACCENT))
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ))
        .padding(padding);

    let inner_area = block.inner(area);
    block.render(area, buf);
    inner_area
}

/// The `❯❯ ` prompt glyph followed by `text_area`, as the search box and the group picker share.
pub fn draw_prompt_row(text_area: &TextArea, area: Rect, buf: &mut Buffer) {
    let [symbol_area, input_area] =
        Layout::horizontal([Constraint::Length(3), Constraint::Fill(1)]).areas(area);

    Paragraph::new(Span::styled(
        "❯❯ ",
        Style::default().fg(theme::KEY).add_modifier(Modifier::BOLD),
    ))
    .render(symbol_area, buf);

    text_area.render(input_area, buf);
}

pub const HIGHLIGHT_SYMBOL: &str = "▌ ";

/// Applies the highlight every table in the UI shares, so they cannot drift apart.
pub fn highlighted_table(table: Table<'_>) -> Table<'_> {
    table
        .row_highlight_style(
            Style::default()
                .bg(theme::SELECTION_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(HIGHLIGHT_SYMBOL)
}

pub fn draw_header(
    active: AppScreen,
    right_text: &str,
    status: Status,
    area: Rect,
    buf: &mut Buffer,
) {
    const TABS: [(AppScreen, &str); 3] = [
        (AppScreen::Targets, "targets"),
        (AppScreen::Settings, "settings"),
        (AppScreen::Logs, "logs"),
    ];

    let mut tabs = Vec::new();
    for (screen, label) in TABS {
        if !tabs.is_empty() {
            tabs.push(Span::styled(" · ", Style::default().fg(theme::MUTED)));
        }
        tabs.push(Span::styled(
            label,
            if screen == active {
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::MUTED)
            },
        ));
    }

    let mut right = vec![Span::styled(
        right_text.to_string(),
        Style::default().fg(theme::MUTED),
    )];
    if status.is_busy() {
        right.push(Span::styled("  ●", Style::default().fg(status.color())));
    }

    draw_split_row(Line::from(tabs), Line::from(right), area, buf);
}

/// `keys` is `(key, label, enabled)`. A key that does nothing here is dimmed rather than dropped,
/// so the bar keeps the same shape from screen to screen.
pub fn draw_footer(
    keys: &[(&str, &str, bool)],
    status: Status,
    update_version: Option<&str>,
    area: Rect,
    buf: &mut Buffer,
) {
    let mut left = Vec::new();
    for (key, label, enabled) in keys {
        if !left.is_empty() {
            left.push(Span::raw("   "));
        }
        left.extend(key_span(key, label, *enabled));
    }

    if let Some(version) = update_version {
        left.push(Span::raw("   "));
        left.extend(key_span("^U", &format!("update v{version}"), true));
    }

    let right = Line::from(vec![
        Span::styled("●", Style::default().fg(status.color())),
        Span::styled(
            format!(" {}", status.label()),
            Style::default().fg(theme::MUTED),
        ),
    ]);

    draw_split_row(Line::from(left), right, area, buf);
}

/// Clamped to `area`, so a terminal smaller than the modal cannot produce an empty or oversized
/// `Rect`.
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let [_, row, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height.min(area.height)),
        Constraint::Fill(1),
    ])
    .areas(area);

    let [_, centred, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width.min(area.width)),
        Constraint::Fill(1),
    ])
    .areas(row);

    centred
}
