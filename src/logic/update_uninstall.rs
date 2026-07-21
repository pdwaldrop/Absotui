use log::{info, error};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time::{timeout, Duration};
use pty_process::Command as PtyCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Update,
    Uninstall,
}

pub enum ProgressEvent {
    Line(String),
    NeedPassword,
    AuthFailed,
    Finished(Result<(), String>),
}

enum UpdateError {
    AuthFailed,
    Other(String),
}

// Runs the same hello_absotui.sh flow `absotui --update`/`--uninstall` already runs
// (see clap.rs's run_hello_absotui), but from inside a live TUI session instead of a
// fresh CLI invocation: authenticate sudo the same way a real terminal would (fingerprint
// reader first if the system has one configured, falling back to a typed password only if
// that doesn't pan out), then drive the script's own interactive prompts with scripted
// stdin answers and stream its output back line by line instead of letting it own the
// terminal directly. Returns a channel to push typed passwords back in on ProgressEvent::
// NeedPassword, since unlike the script phase, we don't know up front whether one will
// even be asked for.
pub fn spawn(action: Action) -> (UnboundedReceiver<ProgressEvent>, UnboundedSender<String>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let (password_tx, password_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        run(action, password_rx, tx).await;
    });
    (rx, password_tx)
}

async fn run(action: Action, mut password_rx: UnboundedReceiver<String>, tx: UnboundedSender<ProgressEvent>) {
    match run_update_or_uninstall(action, &mut password_rx, &tx).await {
        Ok(()) => {
            info!("[update_uninstall] {action:?} finished successfully");
            let _ = tx.send(ProgressEvent::Finished(Ok(())));
        }
        Err(UpdateError::AuthFailed) => {
            let _ = tx.send(ProgressEvent::AuthFailed);
        }
        Err(UpdateError::Other(e)) => {
            error!("[update_uninstall] {action:?} failed: {e}");
            let _ = tx.send(ProgressEvent::Finished(Err(e)));
        }
    }
}

// Same one-liner clap.rs's run_hello_absotui builds (fresh copy of the script from
// `stable`, run it, clean up the tmpfile) - `2>&1` merges stderr into the same ordered
// stream `negotiate` reads below.
fn install_script(action: Action) -> String {
    let command_word = match action {
        Action::Update => "update",
        Action::Uninstall => "uninstall",
    };
    format!(
        r#"tmpfile=$(mktemp) && curl -LsSf https://github.com/pdwaldrop/Absotui/raw/stable/hello_absotui.sh -o "$tmpfile" && bash "$tmpfile" {command_word} 2>&1; rc=$?; rm -f "$tmpfile"; exit $rc"#
    )
}

#[cfg(not(target_os = "macos"))]
async fn run_update_or_uninstall(
    action: Action,
    password_rx: &mut UnboundedReceiver<String>,
    tx: &UnboundedSender<ProgressEvent>,
) -> Result<(), UpdateError> {
    let (mut pty, pts) = pty_process::open().map_err(|e| UpdateError::Other(format!("Couldn't open a pty: {e}")))?;

    let mut validate_cmd = PtyCommand::new("sudo");
    validate_cmd = validate_cmd.args(["-k", "-v"]);
    let mut validate_child = validate_cmd
        .spawn_borrowed(&pts)
        .map_err(|e| UpdateError::Other(format!("Failed to launch sudo: {e}")))?;
    match negotiate(&mut pty, &mut validate_child, password_rx, tx, Duration::from_secs(150), "authenticating").await {
        Ok(status) if status.success() => {}
        Ok(_) => return Err(UpdateError::AuthFailed),
        Err(e) => return Err(UpdateError::Other(e)),
    }

    // Reuses the same pty (`spawn_borrowed` instead of `spawn`, which would consume
    // `pts`) rather than opening a fresh one for the script - sudo's timestamp cache is
    // scoped per controlling-terminal (`tty_tickets`, on by default on most distros), so
    // the credentials just validated above only carry silently into the script's own
    // internal `sudo` calls if both phases share a tty. Confirmed live: without this, a
    // system with a fingerprint reader configured got asked to scan twice - once here,
    // once again partway through the script.
    let mut script_cmd = PtyCommand::new("bash");
    script_cmd = script_cmd.args(["-c", &install_script(action)]);
    let mut script_child = script_cmd
        .spawn_borrowed(&pts)
        .map_err(|e| UpdateError::Other(format!("Failed to launch installer: {e}")))?;

    finish_script(&mut pty, &mut script_child, action, password_rx, tx).await
}

// `spawn_borrowed` (the call above that lets both phases share one pty) isn't available
// on macOS, so each phase gets its own independent pty here instead - same shape this
// feature originally had. A script-internal `sudo` call could still re-prompt there; no
// better or worse than before this file started sharing tickets on other platforms.
#[cfg(target_os = "macos")]
async fn run_update_or_uninstall(
    action: Action,
    password_rx: &mut UnboundedReceiver<String>,
    tx: &UnboundedSender<ProgressEvent>,
) -> Result<(), UpdateError> {
    {
        let (mut pty, pts) = pty_process::open().map_err(|e| UpdateError::Other(format!("Couldn't open a pty: {e}")))?;
        let mut validate_cmd = PtyCommand::new("sudo");
        validate_cmd = validate_cmd.args(["-k", "-v"]);
        let mut validate_child = validate_cmd
            .spawn(pts)
            .map_err(|e| UpdateError::Other(format!("Failed to launch sudo: {e}")))?;
        match negotiate(&mut pty, &mut validate_child, password_rx, tx, Duration::from_secs(150), "authenticating").await {
            Ok(status) if status.success() => {}
            Ok(_) => return Err(UpdateError::AuthFailed),
            Err(e) => return Err(UpdateError::Other(e)),
        }
    }

    let (mut pty, pts) = pty_process::open().map_err(|e| UpdateError::Other(format!("Couldn't open a pty: {e}")))?;
    let mut script_cmd = PtyCommand::new("bash");
    script_cmd = script_cmd.args(["-c", &install_script(action)]);
    let mut script_child = script_cmd
        .spawn(pts)
        .map_err(|e| UpdateError::Other(format!("Failed to launch installer: {e}")))?;

    finish_script(&mut pty, &mut script_child, action, password_rx, tx).await
}

async fn finish_script(
    pty: &mut pty_process::Pty,
    child: &mut tokio::process::Child,
    action: Action,
    password_rx: &mut UnboundedReceiver<String>,
    tx: &UnboundedSender<ProgressEvent>,
) -> Result<(), UpdateError> {
    // Answers the script's own interactive prompts in order: "update to latest? (Y/n)"
    // then "which install method?" for update (always "1" - download the precompiled
    // binary; see the plan's scope decision on why source-method updates stay a manual
    // `absotui --update` thing), or just "uninstall? (Y/n)" for uninstall (we already
    // got real confirmation in our own UI before getting here). Written blind and
    // up front, same as before this file started using a pty: since we already know
    // these answers regardless of what's on screen, there's no need to wait for the
    // prompts to actually appear first.
    let answers = match action {
        Action::Update => "y\n1\n",
        Action::Uninstall => "y\n",
    };
    let _ = pty.write_all(answers.as_bytes()).await;

    let status = negotiate(pty, child, password_rx, tx, Duration::from_secs(180), "running the installer")
        .await
        .map_err(UpdateError::Other)?;
    if status.success() {
        Ok(())
    } else {
        let code = status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        Err(UpdateError::Other(format!("Installer exited with status {code}")))
    }
}

// Shared read/prompt/answer loop for both phases above: streams the child's pty output
// back as ProgressEvent::Line, and if the child's side ever genuinely looks like it's
// waiting on a password (see read_until_idle_or_eof), asks the UI for one and writes
// whatever comes back in, then keeps reading - covers sudo retrying after a wrong
// password same as a real terminal would, not just a single attempt.
async fn negotiate(
    pty: &mut pty_process::Pty,
    child: &mut tokio::process::Child,
    password_rx: &mut UnboundedReceiver<String>,
    tx: &UnboundedSender<ProgressEvent>,
    chunk_timeout: Duration,
    phase: &str,
) -> Result<std::process::ExitStatus, String> {
    let mut partial = String::new();
    let mut buf = [0u8; 1024];

    loop {
        // Only wraps the machine-paced waiting below - once we're actually waiting on
        // the user to type something, there's no clock running (see the
        // `password_rx.recv()` branch further down), so a slow typist can't get cut off.
        match timeout(chunk_timeout, read_until_idle_or_eof(pty, &mut buf, &mut partial, tx)).await {
            Err(_) => {
                let _ = child.start_kill();
                return Err(format!("Timed out {phase}"));
            }
            Ok(Err(e)) => {
                let _ = child.start_kill();
                return Err(format!("Couldn't talk to sudo: {e}"));
            }
            Ok(Ok(true)) => {
                return child.wait().await.map_err(|e| format!("sudo process error: {e}"));
            }
            Ok(Ok(false)) => {
                let _ = tx.send(ProgressEvent::NeedPassword);
                match password_rx.recv().await {
                    Some(password) => {
                        partial.clear();
                        if pty.write_all(format!("{password}\n").as_bytes()).await.is_err() {
                            let _ = child.start_kill();
                            return Err("Couldn't send password to sudo".to_string());
                        }
                    }
                    None => {
                        // Sender dropped - user backed out of the password screen.
                        let _ = child.start_kill();
                        return Err("Cancelled".to_string());
                    }
                }
            }
        }
    }
}

// Reads from the pty until either the child's side closes (Ok(true)) or the current
// unterminated line has gone quiet for a moment (Ok(false)) - the same way tools like
// `expect` tell an info message apart from a real prompt waiting on input. An info
// message (eg. "Place your finger on the fingerprint reader") is newline-terminated, so
// a quiet spell right after one leaves `partial` empty and we just keep waiting - that's
// what covers a fingerprint scan legitimately taking its time. A genuine prompt (eg.
// "[sudo] password for pdwaldrop: ") is never newline-terminated, so a quiet spell with
// something left in `partial` means it's actually waiting on us.
async fn read_until_idle_or_eof(
    pty: &mut pty_process::Pty,
    buf: &mut [u8],
    partial: &mut String,
    tx: &UnboundedSender<ProgressEvent>,
) -> std::io::Result<bool> {
    loop {
        match timeout(Duration::from_millis(500), pty.read(buf)).await {
            Ok(Ok(0)) => return Ok(true),
            Ok(Ok(n)) => {
                partial.push_str(&strip_ansi(&String::from_utf8_lossy(&buf[..n])));
                while let Some(pos) = partial.find('\n') {
                    let line: String = partial.drain(..=pos).collect();
                    let line = line.trim_end_matches(['\r', '\n']);
                    if !line.trim().is_empty() {
                        let _ = tx.send(ProgressEvent::Line(line.to_string()));
                    }
                }
            }
            // EIO on Linux is how a pty master read() reports "the slave side closed" -
            // there's no clean EOF-as-Ok(0) guarantee like a pipe.
            Ok(Err(e)) if e.raw_os_error() == Some(5) => return Ok(true),
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                if !partial.trim().is_empty() {
                    return Ok(false);
                }
            }
        }
    }
}

// hello_absotui.sh's own output is plain multi-line `echo`, except the changelog
// display which uses ANSI dim/reset codes, and one binary-download curl call that can
// emit a `\r`-driven progress meter if its own -s/-sS flags don't suppress it. Strip
// both defensively rather than trusting every code path upstream to stay silent.
fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c2 in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&c2) {
                        break;
                    }
                }
            }
        } else if c != '\r' {
            result.push(c);
        }
    }
    result
}
