use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use std::io::{self, Write};
use std::process;
use crossterm::cursor::Show;
use crate::utils::scroll_wheel::restore_terminal_scroll_wheel;

// exit the app
pub fn clean_exit() {
    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, Show, LeaveAlternateScreen);
    let _ = stdout.flush();
    restore_terminal_scroll_wheel();
    process::exit(0);
}
