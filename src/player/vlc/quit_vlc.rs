use std::net::TcpStream;
use std::io::{self, Write};
use log::{info, error};
use std::process::Command;

// to quit quit VLC with shutdown cmd in cvlc
pub fn quit_vlc(address: &str, port: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(format!("{address}:{port}"))?;

    writeln!(stream, "shutdown")?;

    info!("[quit_vlc.rs] VLC successfully quit");

    Ok(())
}

// need also to pkill VLC for macos - otherwise, issue:
// impossible to launch another track in the same session.
pub fn pkill_vlc() {
    if cfg!(target_os = "macos") {  
        let status = Command::new("pkill")
            .arg("VLC")
            .status()
            .expect("Failed to execute pkill");

        if status.success() {
            info!("VLC pkill success");
        } else {
            error!("VLC pkill error");
        }
    } 
}

