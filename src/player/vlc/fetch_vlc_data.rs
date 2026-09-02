use std::io::{self};
use vlc_rc::Client;
use tokio::net::TcpStream;
use std::process::Command;
use std::str;
use regex::Regex;
use log::{info, warn, error};

/// Polls VLC's RC connection once a second for the current playback position, for as
/// long as `is_vlc_running` reports the port still open.
pub async fn fetch_vlc_data(port: String, address: String) -> Result<Option<u32>, io::Error> {

    loop {
        if !is_vlc_running(port.clone(), address.clone()).await {
            break Ok(None);
        }

        let mut player = match Client::connect(format!("{address}:{port}")) {
            Ok(player) => player,
            Err(e) => {
                error!("[fetch_vlc_data] {e}");
                // Unlike the "no data yet" path below, this had no sleep - a persistent
                // handshake failure (port open but VLC not yet ready to talk RC) busy-
                // looped this task at full CPU instead of backing off.
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };
        let seconds = match player.get_time() {
            Ok(Some(value)) => Some(value),
            Ok(None) => None,
            Err(e) => {
                eprintln!("Failed to fetch time from VLC: {e}");
                error!("Failed to fetch time from VLC: {e}");
                None
            }
        };

        if let Some(sec) = seconds {
            if sec > 0 {
                return Ok(Some(sec));
            }
            info!("[is_vlc_running][check_seconds]: {sec:?}");
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

}

pub async fn fetch_vlc_is_playing(port: String, address: String) -> Result<bool, String> {
    let mut player = match Client::connect(format!("{address}:{port}")) {
        Ok(player) => player,
        Err(e) => {
            warn!("[fetch_vlc_is_playing] Failed to connect to VLC at port {port}: {e}");
            return Err(format!("Failed to connect to VLC at port {port}: {e}"));
        }};

    let is_playing = match player.is_playing() {
        Ok(true) => true,
        // VLC is still open but playback reached the end of the track - distinct from
        // the Err case below, where VLC itself is no longer running.
        Ok(false) => false,
        Err(e) => {
            error!("Failed to check the play status of VLC: {e}");
            return Err(format!("Failed to check the play status of VLC: {e}"))
        }
    };

    Ok(is_playing)
}

pub async fn is_vlc_running(port: String, address: String) -> bool {
    if TcpStream::connect(format!("{address}:{port}")).await.is_ok() {
        true
    } else {
        info!("[is_vlc_running] VLC is not running (port {port} is closed).");
        false
    }
}

pub async fn get_vlc_version() -> Result<String, io::Error> {

    let command: &str = if cfg!(target_os = "macos") {
        "/Applications/VLC.app/Contents/MacOS/VLC"
    } else {
        "vlc"
    };

    let output = Command::new(command)
        .arg("--version")
        .output()?;

    let version_output = str::from_utf8(&output.stdout)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?; 

    let re = Regex::new(r"VLC (?:media player |version )?([\d.]+)").unwrap();

    if let Some(captures) = re.captures(version_output)
        && let Some(version) = captures.get(1) {
            return Ok(version.as_str().to_string());
        }

    Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Could not extract VLC version",
    ))
}
