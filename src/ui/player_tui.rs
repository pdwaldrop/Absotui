use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use crate::db::crud::get_is_show_key_bindings;


pub fn render_player(area: Rect, buf: &mut ratatui::buffer::Buffer, player_info: Vec<String>, bg_color: Vec<u8>, progress_bar_color: Vec<u8>, username: &str) {
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

    let block_width = area.width;
    let new_y = area.y + area.height.saturating_sub(9); // the line number where player start
    let block_height = 4; // number of line of the player (in lines)

    // Create the background block with background color
    let bg_color_player = Color::Rgb(bg_color[0], bg_color[1], bg_color[2]);
    let block_area = Rect::new(area.x, new_y, block_width, block_height);
    let block = Block::default()
        .style(Style::default().bg(bg_color_player));

    // Text area
    let text_area_width = block_width - 6;
    let text_area_x = (area.width.saturating_sub(text_area_width)) / 2; // Center the text
    let text_area = Rect::new(text_area_x, new_y, text_area_width, block_height);

    // Split into: blank spacer line, title line (gets the progress fill), details+key bindings
    let [spacer_area, title_area, rest_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
    ]).areas(text_area);
    let _ = spacer_area; // already covered by the block's background tint below

    let mut key_bindings = String::new();
    let is_show_key_bindings = get_is_show_key_bindings(username);
    if is_show_key_bindings == "1" {
        key_bindings = "Spc: pause/play | p/u: +/−10s | P/U: nxt/prev ch. | O/I: spd +/− | o/i: vol +/− | T: real/content time | Y: quit".to_string();
    }

    let progress_color = Color::Rgb(progress_bar_color[0], progress_bar_color[1], progress_bar_color[2]);

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
        Span::styled(vol_filled, Style::default().underline_color(progress_color).add_modifier(Modifier::UNDERLINED)),
        Span::raw(vol_unfilled),
    ]);

    // Create the paragraph for the details/key-bindings lines (title line is handled separately below)
    let paragraph = Paragraph::new(vec![details_line, Line::from(key_bindings)])
        .centered()
        .block(Block::default());

    // Render the background block first, then the details paragraph on top of it
    block.render(block_area, buf);
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
        Span::styled(" ".repeat(padding_left), Style::default().bg(bg_color_player)),
        Span::styled(filled_text, Style::default().bg(progress_color)),
        Span::styled(unfilled_text, Style::default().bg(bg_color_player)),
    ]);
    Paragraph::new(title_line).render(title_area, buf);
}
