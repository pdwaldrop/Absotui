use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Block;

// Named ANSI colors, not RGB - a terminal remaps what each of these actually
// renders as based on its own theme, so the accent stays "in theme" instead of
// fighting it the way a fixed RGB value would.
pub const ACCENT_STRUCTURE: Color = Color::Blue;
pub const ACCENT_ACTIVE: Color = Color::Green;
pub const ACCENT_KEY: Color = Color::Yellow;
pub const ACCENT_ERROR: Color = Color::Red;

/// Standard bordered/titled box used for every section (list, info,
/// description, header panels).
pub fn section_block(title: &str) -> Block<'static> {
    Block::bordered()
        .title(title.to_string())
        .border_style(Style::new().fg(ACCENT_STRUCTURE))
}

/// One keybind chip: `keys` rendered as a reverse-video highlighted block,
/// followed by a space and `desc` in plain text - replaces the old
/// "key: description" convention (no colon).
pub fn footer_hint(keys: &str, desc: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(format!(" {keys} "), Style::default().fg(ACCENT_KEY).add_modifier(Modifier::REVERSED)),
        Span::raw(" "),
        Span::raw(desc.to_string()),
    ]
}

/// Joins hints into one line, 2 spaces between chips.
pub fn footer_line(hints: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (keys, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.extend(footer_hint(keys, desc));
    }
    Line::from(spans)
}

/// Builds the full (possibly multi-line) footer text from an ordered list of
/// hint lines.
pub fn footer_text(lines: &[Vec<(&str, &str)>]) -> Text<'static> {
    Text::from(lines.iter().map(|hints| footer_line(hints)).collect::<Vec<_>>())
}
