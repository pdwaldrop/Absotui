use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// Converts a description/subtitle string that may contain HTML markup (as sent by
/// Audiobookshelf for some libraries/podcasts) into styled lines: `<b>`/`<strong>` and
/// `<i>`/`<em>` become real bold/italic spans, `<br>`/`<p>`/`<li>` become line breaks
/// (with `<li>` prefixed by a bullet), and every other tag is just dropped. Unclosed tags
/// degrade gracefully - worst case a style "leaks" to the end of the text - rather than
/// panicking, since real-world descriptions occasionally have sloppy markup.
pub fn html_to_lines(input: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut text = String::new();
    let mut bold = false;
    let mut italic = false;

    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c != '<' {
            text.push(c);
            continue;
        }

        let mut tag = String::new();
        for tc in chars.by_ref() {
            if tc == '>' {
                break;
            }
            tag.push(tc);
        }
        let tag_lower = tag.to_lowercase();
        let is_closing = tag_lower.starts_with('/');
        let tag_name = tag_lower.trim_start_matches('/').split_whitespace().next().unwrap_or("");

        match tag_name {
            "b" | "strong" => {
                flush_span(&mut text, &mut spans, bold, italic);
                bold = !is_closing;
            }
            "i" | "em" => {
                flush_span(&mut text, &mut spans, bold, italic);
                italic = !is_closing;
            }
            "br" => flush_line(&mut text, &mut spans, &mut lines, bold, italic),
            "p" | "div" if is_closing => {
                flush_line(&mut text, &mut spans, &mut lines, bold, italic);
                lines.push(Line::default());
            }
            "li" if !is_closing => text.push_str("- "),
            "li" => flush_line(&mut text, &mut spans, &mut lines, bold, italic),
            _ => {}
        }
    }
    flush_line(&mut text, &mut spans, &mut lines, bold, italic);

    // Adjacent closing </p>/</div> tags each add a blank separator line - collapse any
    // run of them, and drop one trailing blank line left by a final closing tag.
    lines.dedup_by(|a, b| a.width() == 0 && b.width() == 0);
    while lines.len() > 1 && lines.last().is_some_and(|l| l.width() == 0) {
        lines.pop();
    }

    lines
}

fn flush_span(text: &mut String, spans: &mut Vec<Span<'static>>, bold: bool, italic: bool) {
    if text.is_empty() {
        return;
    }
    let decoded = decode_entities(text);
    text.clear();

    let mut style = Style::default();
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    spans.push(Span::styled(decoded, style));
}

fn flush_line(text: &mut String, spans: &mut Vec<Span<'static>>, lines: &mut Vec<Line<'static>>, bold: bool, italic: bool) {
    flush_span(text, spans, bold, italic);
    lines.push(Line::from(std::mem::take(spans)));
}

fn decode_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}
