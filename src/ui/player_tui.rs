use ratatui::{
    layout::Rect,         
    style::{Color, Style},  
    widgets::{Block, Paragraph, Widget},
};
use crate::db::crud::get_is_show_key_bindings;


pub fn render_player(area: Rect, buf: &mut ratatui::buffer::Buffer, player_info: Vec<String>, bg_color: Vec<u8>, username: &str) {
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


    let mut key_bindings = String::new();
    let is_show_key_bindings = get_is_show_key_bindings(username);
    if is_show_key_bindings == "1" {
        key_bindings = "Spc: pause/play | p/u: +/−10s | P/U: nxt/prev ch. | O/I: spd +/− | o/i: vol +/− | Y: quit".to_string();
    }  

    // Create the paragraph
    let paragraph = Paragraph::new(format!(
            "\n{} by {} | {} \n {} {} / {} | Elapsed: {} | Left: {} ({}%) | Speed: {}x\n{}", 
            player_info[0], // Title
            player_info[1], // Author
            player_info[2], // Chapter
            match player_info[3].as_str() {
                "false" => "⏸".to_string(),
                "true" => "▶".to_string(),
                _ => String::new(),

            },
            player_info[4], // Current time
            player_info[5], // Total duration
            player_info[6], // Elapsed time
            player_info[7], // Remaining time
            player_info[8], // Percent progress
            player_info[9], // Speed rate
            key_bindings
    ))
        .centered()
        .block(Block::default());

    // Render the paragraph and background block
    paragraph.render(text_area, buf);
    block.render(block_area, buf);
}

