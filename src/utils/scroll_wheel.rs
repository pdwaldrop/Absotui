use std::io::{self, Write};

/// Disables "alternate scroll mode" (xterm/VTE `?1007`) - the terminal-level behavior,
/// independent of this app's own input handling (it has none for the mouse), where a
/// terminal translates wheel/trackpad scroll into Up/Down key sequences while an app
/// is on the alternate screen. Without this, scrolling reaches `App::handle_key`
/// indistinguishable from real arrow key presses, moving the list selection - a
/// two-finger trackpad scroll fires far more of these per inch than a wheel notch,
/// which is what makes the selector appear to run away. Terminals that don't support
/// the sequence just ignore it.
pub fn disable_terminal_scroll_wheel() {
    let mut stdout = io::stdout();
    let _ = write!(stdout, "\x1b[?1007l");
    let _ = stdout.flush();
}

/// Restores alternate scroll mode before handing the terminal back to the shell, so
/// wheel/trackpad scroll keeps working as expected outside the app (shell scrollback,
/// `less`, etc).
pub fn restore_terminal_scroll_wheel() {
    let mut stdout = io::stdout();
    let _ = write!(stdout, "\x1b[?1007h");
    let _ = stdout.flush();
}
