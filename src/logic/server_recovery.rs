use crate::app::App;
use crate::config::load_config;
use crate::db::crud::{select_default_usr, update_server_address};
use crate::utils::exit_app::clean_exit;
use color_eyre::eyre::{Report, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use log::error;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::DefaultTerminal;
use ratatui_textarea::TextArea;

enum Action {
    Retry,
    ChangeAddress(String),
    Quit,
    Cancel,
}

fn classify_error(report: &Report) -> String {
    if let Some(e) = report.downcast_ref::<reqwest::Error>()
        && (e.is_connect() || e.is_timeout() || e.is_request())
    {
        return format!("Could not reach the server: {e}");
    }
    format!("Reached the server, but something went wrong: {report}")
}

/// Runs `App::new()` in a retry loop, showing a recovery screen on failure
/// (Retry / change server address / Quit) instead of letting the error
/// propagate out of `main` and silently kill the process.
///
/// `allow_cancel` adds an Esc-to-cancel option, returning `Ok(None)`. Only
/// meaningful at the two mid-session reinit call sites (`R`-refresh, library
/// switch) where a working `App` already exists to fall back to - startup
/// has none, so it passes `false` and never sees `Action::Cancel`.
pub async fn init_app_with_retry(terminal: &mut DefaultTerminal, allow_cancel: bool) -> Result<Option<App>> {
    loop {
        match App::new().await {
            Ok(app) => return Ok(Some(app)),
            Err(report) => {
                error!("[init_app_with_retry] {report}");
                let default_usr = select_default_usr().unwrap_or_default();
                let username = default_usr.first().cloned().unwrap_or_default();
                let address = default_usr.get(1).cloned().unwrap_or_default();
                let message = classify_error(&report);

                match render_error_screen(terminal, &address, &message, allow_cancel)? {
                    Action::Retry => continue,
                    Action::ChangeAddress(new_address) => {
                        // The saved token was issued by the old server, so this alone
                        // won't fix auth against a different one - the next retry will
                        // likely surface as a "reached it, but something's wrong" error,
                        // which is expected (full re-login is a quit + AppLogin away).
                        let _ = update_server_address(new_address.trim(), &username);
                        continue;
                    }
                    Action::Quit => clean_exit(), // never returns
                    Action::Cancel => return Ok(None),
                }
            }
        }
    }
}

/// Blocking message/action screen, modeled on `logic::auth::auth_input::auth`'s
/// event loop. `[A]` drops into a one-field address-edit sub-view prefilled
/// with the current address; `Esc` there backs out without committing.
fn render_error_screen(
    terminal: &mut DefaultTerminal,
    address: &str,
    message: &str,
    allow_cancel: bool,
) -> Result<Action> {
    let config = load_config()?;
    let fg = config.colors.login_foreground_color.clone();
    let bg = config.colors.log_background_color.clone();

    let mut editing_address = false;
    let mut textarea = TextArea::from(vec![address.to_string()]);
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("New server address")
            .border_style(Style::default().fg(Color::Rgb(fg[0], fg[1], fg[2]))),
    );
    textarea.set_placeholder_text("http:// or https:// required");

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let background = Block::default()
                .style(Style::default().bg(Color::Rgb(bg[0], bg[1], bg[2])));
            frame.render_widget(background, area);

            if editing_address {
                let input_area = Rect {
                    x: (area.width / 4).max(1),
                    y: area.height.saturating_sub(3) / 2,
                    width: (area.width / 2).max(20),
                    height: 3,
                };
                frame.render_widget(&textarea, input_area);
            } else {
                let mut lines = vec![
                    Line::from("Couldn't reach the Audiobookshelf server"),
                    Line::from(format!("Server: {address}")),
                    Line::from(""),
                    Line::from(message.to_string()),
                    Line::from(""),
                ];
                let mut hint = String::from("[R] Retry   [A] Change server address   [Q] Quit");
                if allow_cancel {
                    hint.push_str("   [Esc] Keep using current data");
                }
                lines.push(Line::from(hint));

                let paragraph = Paragraph::new(lines)
                    .wrap(Wrap { trim: true })
                    .style(Style::default().fg(Color::Rgb(fg[0], fg[1], fg[2])))
                    .block(Block::default().borders(Borders::ALL).title("Connection error"));

                let msg_area = Rect {
                    x: (area.width / 8).max(1),
                    y: (area.height / 4).max(1),
                    width: (area.width * 3 / 4).max(20),
                    height: (area.height / 2).max(9),
                };
                frame.render_widget(paragraph, msg_area);
            }
        })?;

        let ev = event::read()?;
        if editing_address {
            match ev {
                Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                    let new_address = textarea.lines().join("\n");
                    return Ok(Action::ChangeAddress(new_address));
                }
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                    editing_address = false;
                }
                Event::Key(input) => {
                    textarea.input(input);
                }
                _ => {}
            }
        } else {
            match ev {
                Event::Key(KeyEvent { code: KeyCode::Char('r' | 'R'), .. }) => return Ok(Action::Retry),
                Event::Key(KeyEvent { code: KeyCode::Char('a' | 'A'), .. }) => {
                    editing_address = true;
                }
                Event::Key(KeyEvent { code: KeyCode::Char('q' | 'Q'), .. }) => return Ok(Action::Quit),
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) if allow_cancel => return Ok(Action::Cancel),
                _ => {}
            }
        }
    }
}
