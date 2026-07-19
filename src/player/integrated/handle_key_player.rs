use std::io::{self, Write};
use std::net::TcpStream;
use crate::db::crud::{get_listening_session, update_is_playback, update_speed_rate, get_speed_rate, update_volume_up, update_volume_down, get_is_per_item_speed, get_item_speed_rate, set_item_speed_rate, update_item_speed_rate};
use std::thread;
use std::time::Duration;

pub fn handle_key_player(key: &str, address: &str, port: &str, is_playback: &mut bool, username: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(format!("{address}:{port}"))?;

    let jump = "10";

    match key {
        // toggle playback/pause
        " " => {
            match get_listening_session() {
                Ok(Some(session)) => {
                    if session.is_playback {
                    let _ = update_is_playback("0", session.id_session.as_str());
                    } else {
                    let _ = update_is_playback("1", session.id_session.as_str());
                    }
                }
                Ok(None) => {

                }
                Err(_e) => {

                }
            }
            if *is_playback {
                writeln!(stream, "pause")?;
            } else {
                writeln!(stream, "play")?;
            }
            *is_playback = !*is_playback;
        }

        // For some cmd, below, need pause => cmd => play
        // Allow vlc buffer issue. 
        // Futhermore, need a thread for macos otherwise vlc buffer issue
        // Otherwise buffer issue and the player freeze
        // But maybe it's not necessary because I test absotui on macos with a VM
        // and maybe the VM add a little delay.. but for now I try like this

        // jump forward
        "p" => {
            writeln!(stream, "pause")?; 
            writeln!(stream, "seek +{jump}")?;
            if cfg!(target_os = "macos") {
            thread::sleep(Duration::from_millis(500));
            }
            writeln!(stream, "play")?;
        }
        // jump backward
        "u" => {
            writeln!(stream, "pause")?;
            writeln!(stream, "seek -{jump}")?;
            if cfg!(target_os = "macos") {
            thread::sleep(Duration::from_millis(500));
            }
            writeln!(stream, "play")?;
        }
        // next chapter
        "P" => {
            writeln!(stream, "pause")?;
            writeln!(stream, "chapter_n")?;
            if cfg!(target_os = "macos") {
            thread::sleep(Duration::from_millis(500));
            }
            writeln!(stream, "play")?;
        }
        // previous chapter
        "U" => {
            writeln!(stream, "pause")?;
            writeln!(stream, "chapter_p")?;
            if cfg!(target_os = "macos") {
            thread::sleep(Duration::from_millis(500));
            }
            writeln!(stream, "play")?;
        }
        // volume up
        "o" => {
            if let Ok(Some(session)) = get_listening_session() {
                let _ = update_volume_up(session.id_session.as_str());
            }
            writeln!(stream, "volup")?;
        }
        // volume down
        "i" => {
            if let Ok(Some(session)) = get_listening_session() {
                let _ = update_volume_down(session.id_session.as_str());
            }
            writeln!(stream, "voldown")?;
        }
        // speed rate up
        "O" => {
            let speed_rate = adjust_speed_rate(username, true);
            writeln!(stream, "rate {speed_rate}")?;
        }
        // speed rate down
        "I" => {
            let speed_rate = adjust_speed_rate(username, false);
            writeln!(stream, "rate {speed_rate}")?;
        }
        // shutdown
        "Y" => {
            writeln!(stream, "shutdown")?;
        }
        _ => {}
    }

    Ok(())
}

// Adjusts and returns the effective speed rate: per-item (keyed by the active
// session's id_item) if Settings > Per-Item Speed is on, otherwise the single shared
// speed_rate - same resolution start_vlc uses when launching a new session. Seeds the
// item's row at a fixed 1.0x baseline on first touch (not the current global rate -
// see update_is_per_item_speed/resolve_speed_rate), whether that's because this is the
// item's first playback under per-item mode, or because per-item mode was just turned
// on mid-session for an item that hasn't been touched yet.
fn adjust_speed_rate(username: &str, is_up: bool) -> String {
    if get_is_per_item_speed(username) == "1"
        && let Ok(Some(session)) = get_listening_session() {
            if get_item_speed_rate(username, &session.id_item).is_none() {
                let _ = set_item_speed_rate(username, &session.id_item, 1.0);
            }
            let _ = update_item_speed_rate(username, &session.id_item, is_up);
            if let Some(rate) = get_item_speed_rate(username, &session.id_item) {
                return format!("{rate:.1}");
            }
    }
    let _ = update_speed_rate(username, is_up);
    get_speed_rate(username)
}

// Seeks the currently-playing VLC session directly to an absolute position (in seconds) -
// used to jump to a specific chapter's start time. Bracketed by pause/play like the
// relative seeks above, to avoid the same VLC buffering issue.
pub fn seek_to_absolute_time(address: &str, port: &str, seconds: u32) -> io::Result<()> {
    let mut stream = TcpStream::connect(format!("{address}:{port}"))?;
    writeln!(stream, "pause")?;
    writeln!(stream, "seek {seconds}")?;
    if cfg!(target_os = "macos") {
        thread::sleep(Duration::from_millis(500));
    }
    writeln!(stream, "play")?;
    Ok(())
}

