use std::io::{Result, Stdout};
use crossterm::{
    execute,
    style::{Color, SetBackgroundColor},
    terminal, cursor,
};

// pop up message
pub fn pop_message(stdout: &mut Stdout, lines_from_bottom: u16, message: &str) -> Result<()> {
    let (_cols, rows) = terminal::size()?;
    let target_row = rows.saturating_sub(lines_from_bottom);

    execute!(
        stdout,
        cursor::MoveTo(0, target_row),
        SetBackgroundColor(Color::Reset),

    )?;

    println!("{message}");

    Ok(())
}



// to clear a pop up message
pub fn clear_message(stdout: &mut Stdout, lines_from_bottom: u16) -> Result<()> {
    let (_cols, rows) = terminal::size()?;
    let target_row = rows.saturating_sub(lines_from_bottom);

    execute!(
        stdout,
        cursor::MoveTo(0, target_row),
        SetBackgroundColor(Color::Reset),
        terminal::Clear(terminal::ClearType::CurrentLine),
    )?;

    Ok(())
}
