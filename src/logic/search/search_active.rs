use crate::App;
use crate::app::AppView;
use crate::db::crud::get_is_vlc_running;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, Borders};
use ratatui::style::Style;
use ratatui::Terminal;
use std::io;
use ratatui_textarea::{Input, Key, TextArea};
use ratatui::layout::Rect;
use crate::ui::theme;


impl App {
    pub fn search_active(&mut self) -> io::Result<String> {
        let stdout = io::stdout();
        let stdout = stdout.lock();

        let backend = CrosstermBackend::new(stdout);
        let mut term = Terminal::new(backend)?;

        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
            .borders(Borders::ALL)
            .title("Search")
            .border_style(Style::new().fg(theme::ACCENT_STRUCTURE))
        );

        let size = term.size()?;
        // This box draws through its own separate `Terminal` (see the module doc comment
        // in main.rs's render loop), so it has no idea what's on screen below it - it just
        // has to not land on top of it. The player bar (when a session is active) reserves
        // 6 rows for its own box plus a 1-row gap above the footer (see player_tui.rs's
        // `new_y` and `standard_layout`'s `player_gap`/`refresh` constraints); anchoring
        // purely off "3 rows above the footer" without checking for that put this box
        // squarely inside the player's box instead of above it.
        let player_reserved = if get_is_vlc_running(&self.username) == "1" { 7 } else { 0 };
        let search_area = Rect {
            x: 1,
            y: size.height.saturating_sub(5 + player_reserved),
            width: size.width.saturating_sub(2),
            height: 3,
        };

        loop {

            term.draw(|f| {
                f.render_widget(&textarea, search_area);
            })?;
            match crossterm::event::read()?.into() {
                Input { key: Key::Enter, .. } => {
                    self.search_mode = false;
                    self.search_query = textarea.lines().join("\n");
                    self.view_state = AppView::SearchBook;
                    self.list_state_search_results.select(Some(0));
                    break;
                }
                Input { key: Key::Esc, .. } => {
                    self.search_mode = false;
                    break;
                }
                input => {
                    textarea.input(input);
                }
            }
        }
        term.draw(|f| {
            f.render_widget(Block::default(), search_area);
        })?;

        Ok(textarea.lines().join("\n"))

    }
}
