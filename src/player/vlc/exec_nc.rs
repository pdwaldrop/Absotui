use std::io;
use std::process::Command;
use std::process::Output;
use log::error;

// Only called when Settings has `cvlc_term` on - the "open a terminal so cvlc's raw RC
// port can be controlled directly" feature. Called from an un-awaited tokio::spawn task,
// so the old .expect() here panicked that task silently (no user-visible error, no log)
// if `kitty` isn't installed - a totally silent no-op instead of the feature just not
// working visibly.
pub async fn exec_nc(port: &str, address: String) -> io::Result<Output> {
    Command::new("kitty")
        .arg("nc")
        .arg(&address)
        .arg(port)
        .output()
        .inspect_err(|e| error!("[exec_nc] Failed to execute kitty: {e}"))
}

