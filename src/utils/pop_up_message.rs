use std::io::{Result, Stdout};
use crossterm::{
    execute,
    style::{Color, SetBackgroundColor},
    terminal, cursor,
};
use crate::config::load_config;

// pop up message
pub fn pop_message(stdout: &mut Stdout, lines_from_bottom: u16, message: &str) -> Result<()> {
    // import backgorund color - falls back to black rather than an empty Vec (which
    // would panic below on color[0]) if config.toml is unreadable/malformed right now,
    // eg. mid-edit or mid-update.
    let mut color = vec![0, 0, 0];
    if let Ok(cfg) = load_config()
        && cfg.colors.background_color.len() == 3 {
        color = cfg.colors.background_color;
    }

    let (_cols, rows) = terminal::size()?; 
    let target_row = rows.saturating_sub(lines_from_bottom);
    let bg_color = Color::Rgb { r: color[0], g: color[1], b: color[2] };

    execute!(
        stdout,
        cursor::MoveTo(0, target_row), 
        SetBackgroundColor(bg_color),

    )?;

    println!("{message}");

    Ok(())
}



// to clear a pop up message
pub fn clear_message(stdout: &mut Stdout, lines_from_bottom: u16) -> Result<()> {
    // import backgorund color - falls back to black rather than an empty Vec (which
    // would panic below on color[0]) if config.toml is unreadable/malformed right now,
    // eg. mid-edit or mid-update.
    let mut color = vec![0, 0, 0];
    if let Ok(cfg) = load_config()
        && cfg.colors.background_color.len() == 3 {
        color = cfg.colors.background_color;
    }
    let (_cols, rows) = terminal::size()?; 
    let target_row = rows.saturating_sub(lines_from_bottom);
    let bg_color = Color::Rgb { r: color[0], g: color[1], b: color[2] };


    execute!(
        stdout,
        cursor::MoveTo(0, target_row), 
        SetBackgroundColor(bg_color),
        terminal::Clear(terminal::ClearType::CurrentLine), 
    )?;

    Ok(())
}

