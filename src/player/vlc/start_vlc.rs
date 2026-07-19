use std::process::Command;
use std::process::Output;
use crate::db::crud::{get_speed_rate, get_is_per_item_speed, get_item_speed_rate, set_item_speed_rate};

// Resolves the playback speed to launch VLC with: per-item if Settings > Per-Item
// Speed is on, otherwise the single shared speed_rate - unchanged from before this
// setting existed. `id_item` is a book's id, or a podcast show's own id (shared across
// its episodes). A per-item item that hasn't been touched yet always starts at a fixed
// 1.0x baseline, not whatever the shared speed happens to be - turning Per-Item Speed
// on already wipes every existing per-item row for exactly this reason (see
// update_is_per_item_speed), so this only matters for an item played for the first
// time since then.
fn resolve_speed_rate(username: &str, id_item: &str) -> String {
    if get_is_per_item_speed(username) != "1" {
        return get_speed_rate(username);
    }

    if let Some(rate) = get_item_speed_rate(username, id_item) {
        return format!("{rate:.1}");
    }

    let _ = set_item_speed_rate(username, id_item, 1.0);
    "1.0".to_string()
}

pub async fn start_vlc(
    current_time: &String,
    port: &str,
    address: String,
    content_url: &String,
    token: Option<&String>,
    title: String,
    subtitle: String,
    author: String,
    server_address: String,
    program: String,
    username: String,
    id_item: String,
) -> Output {

    let speed_rate = resolve_speed_rate(username.as_str(), id_item.as_str());

    let output: Output = Command::new(&program)
        .arg("-I") // for macos
        .arg("dummy") // for macos
        .arg(format!("--start-time={current_time}"))
        .arg("--extraintf")
        .arg("rc")
        .arg("--rc-host")
        .arg(format!("{address}:{port}"))
        .arg(format!("{}{}?token={}", server_address, content_url, token.unwrap()))
        .arg("--rate")
        .arg(speed_rate)
        .arg("--meta-description")
        .arg(author)
        .arg("--meta-title")
        .arg(subtitle)
        .arg("--meta-artist")
        .arg(title)
        .output()
        .expect("Failed to execute program");

    output
}

