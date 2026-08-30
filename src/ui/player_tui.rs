use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use crate::db::crud::get_is_show_key_bindings;
use crate::ui::theme;

// Shared with tui.rs's Keymap screen (`keymap_entries()`) so this list is only
// ever spelled out once - the player's own control keys aren't tied to any
// particular AppView (the player overlay renders on every screen), so Keymap
// includes these regardless of which screen it was opened from.
pub const PLAYER_KEYS: &[(&str, &str)] = &[
    ("Spc", "pause/play"), ("p/u", "+/−10s"), ("P/U", "nxt/prev ch."),
    ("O/I", "spd +/−"), ("o/i", "vol +/−"), ("T", "real/content time"), ("Y", "quit"),
];

pub fn render_player(area: Rect, buf: &mut ratatui::buffer::Buffer, player_info: Vec<String>, username: &str) {
    // player_info() only pushes the full 12 fields this function indexes into on a
    // successful `Ok(Some(session))` read (see src/player/integrated/player_info.rs) -
    // a transient sqlite read error (Ok(None)/Err path, only 4 fields) shouldn't be
    // able to happen anymore now that get_listening_session has a busy_timeout, but
    // indexing a caller-supplied Vec without checking its length first is fragile
    // regardless of how rare that's supposed to be - skip this frame instead of
    // panicking the whole render loop.
    if player_info.len() < 12 {
        log::error!("render_player: player_info has {} fields, need 12 - skipping this frame", player_info.len());
        return;
    }

    // This box's position is independent of the App's own layout (main.rs renders it
    // before `frame.render_widget(&mut app, ...)`, using the raw frame area, not
    // tui.rs's `standard_layout` split) - it has to stay aligned with that layout's
    // reserved player-gap by hand. That gap sits right after `main_area`, which is
    // `Constraint::Fill(1)` and so grows to absorb whatever the footer *doesn't* need -
    // the footer is 1-2 rows depending on how much it wraps at the current width (see
    // `standard_layout`'s dynamic `footer_height`, floored at 1). Anchoring to the
    // worst case (footer_height=1, meaning `main_area` is at its largest) is what
    // makes this safe for every width: at footer_height=2 this leaves one harmless
    // extra blank row above the box instead of encroaching into `main_area` and
    // getting the app's own next frame to silently paint over this box's top border/
    // title (confirmed live: exactly what happened at footer_height=1 when this was 9).
    let new_y = area.y + area.height.saturating_sub(8); // the line number where player start
    let block_height = 6; // 4 content rows (spacer/title/details/key-bindings) + border top/bottom

    // Full width, matching every other panel's box - no inset margin.
    let text_area = Rect::new(area.x, new_y, area.width, block_height);

    // Its own box in the active accent color (matching the play icon/progress fill),
    // distinct from every other panel's structural-accent box.
    let block = theme::section_block("Now Playing").border_style(Style::new().fg(theme::ACCENT_ACTIVE));
    let inner = block.inner(text_area);
    block.render(text_area, buf);

    // Split into: blank spacer line, title line (gets the progress fill), details+key bindings
    let [spacer_area, title_area, rest_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
    ]).areas(inner);
    let _ = spacer_area; // left blank - nothing to render there

    let mut key_bindings_line = Line::default();
    let is_show_key_bindings = get_is_show_key_bindings(username);
    if is_show_key_bindings == "1" {
        key_bindings_line = theme::footer_line(PLAYER_KEYS);
    }

    // Volume indicator: a subtle underline beneath "Vol NN%" itself, filled up to
    // volume/200 - same underline-fill convention already used for the time/progress
    // text elsewhere, just applied to this short label instead of a separate bar/row.
    // 200 (not 100) because VLC's own volume can be amplified up to double its normal
    // unamplified level (100%) - see update_volume_up/down.
    let volume: i32 = player_info[11].parse().unwrap_or(100);
    let vol_label = format!("Vol {volume}%");
    let vol_chars: Vec<char> = vol_label.chars().collect();
    let vol_fill = (((volume as f32 / 200.0) * vol_chars.len() as f32).round() as usize).min(vol_chars.len());
    let vol_filled: String = vol_chars[..vol_fill].iter().collect();
    let vol_unfilled: String = vol_chars[vol_fill..].iter().collect();

    let details_line = Line::from(vec![
        Span::raw(format!(
            " {} {} / {} | Session: {} | Left: {} ({}%) | Speed: {}x [{}] | ",
            match player_info[3].as_str() {
                "false" => "⏸".to_string(),
                "true" => "▶".to_string(),
                _ => String::new(),

            },
            player_info[4], // Current time
            player_info[5], // Total duration
            player_info[6], // Session time so far (Real or Content, depending on the T toggle) - resets each playback session
            player_info[7], // Remaining time (Real or Content, depending on the T toggle)
            player_info[8], // Percent progress
            player_info[9], // Speed rate
            player_info[10], // "Real" or "Content" mode indicator
        )),
        Span::styled(vol_filled, Style::default().add_modifier(Modifier::UNDERLINED)),
        Span::raw(vol_unfilled),
    ]);

    // Create the paragraph for the details/key-bindings lines (title line is handled separately below)
    let paragraph = Paragraph::new(vec![details_line, key_bindings_line])
        .centered()
        .block(Block::default());

    paragraph.render(rest_area, buf);

    // Title line: progress bar rendered as a background fill directly behind the text,
    // rather than a separate bar, to avoid spending extra vertical space in an already
    // compact player bar.
    // Podcasts store "Episode Title | Podcast Title" directly as the title with an empty
    // author/chapter (see handle_l_pod_home.rs/handle_l_pod.rs), so there's nothing to
    // append here for them - books show "Chapter #. Name | Book" instead.
    let title_text = if player_info[1].is_empty() && player_info[2].is_empty() {
        player_info[0].clone()
    } else {
        format!("{} | {}", player_info[2], player_info[0])
    };
    let percent: f32 = player_info[8].parse().unwrap_or(0.0);

    let chars: Vec<char> = title_text.chars().collect();
    let padding_left = (title_area.width as usize).saturating_sub(chars.len()) / 2;
    let fill_count = (((percent / 100.0) * chars.len() as f32).round() as usize).min(chars.len());
    let filled_text: String = chars[..fill_count].iter().collect();
    let unfilled_text: String = chars[fill_count..].iter().collect();

    let title_line = Line::from(vec![
        Span::raw(" ".repeat(padding_left)),
        Span::styled(filled_text, Style::default().fg(theme::ACCENT_ACTIVE).add_modifier(Modifier::REVERSED)),
        Span::raw(unfilled_text),
    ]);
    Paragraph::new(title_line).render(title_area, buf);
}
