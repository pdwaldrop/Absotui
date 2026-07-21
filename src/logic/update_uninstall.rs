use std::process::Stdio;
use log::{info, error};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time::{timeout, Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Update,
    Uninstall,
}

pub enum ProgressEvent {
    Line(String),
    AuthFailed,
    Finished(Result<(), String>),
}

// Runs the same hello_absotui.sh flow `absotui --update`/`--uninstall` already runs
// (see clap.rs's run_hello_absotui), but from inside a live TUI session instead of a
// fresh CLI invocation: pre-authenticate sudo with the password collected in-app so
// the script's own `sudo` calls proceed silently on the cached ticket, then drive its
// interactive prompts with scripted stdin answers and stream its output back line by
// line instead of letting it own the terminal directly.
pub fn spawn(action: Action, password: String) -> UnboundedReceiver<ProgressEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        run(action, password, tx).await;
    });
    rx
}

async fn run(action: Action, password: String, tx: UnboundedSender<ProgressEvent>) {
    match timeout(Duration::from_secs(30), authenticate_sudo(&password)).await {
        Ok(Ok(true)) => {}
        Ok(Ok(false)) => {
            let _ = tx.send(ProgressEvent::AuthFailed);
            return;
        }
        Ok(Err(e)) => {
            error!("[update_uninstall] sudo auth error: {e}");
            let _ = tx.send(ProgressEvent::Finished(Err(format!("Couldn't run sudo: {e}"))));
            return;
        }
        Err(_) => {
            let _ = tx.send(ProgressEvent::Finished(Err("Timed out waiting for sudo".to_string())));
            return;
        }
    }

    let result = timeout(Duration::from_secs(180), run_script(action, &tx)).await;
    match result {
        Ok(Ok(())) => {
            info!("[update_uninstall] {action:?} finished successfully");
            let _ = tx.send(ProgressEvent::Finished(Ok(())));
        }
        Ok(Err(e)) => {
            error!("[update_uninstall] {action:?} failed: {e}");
            let _ = tx.send(ProgressEvent::Finished(Err(e)));
        }
        Err(_) => {
            let message = "Timed out - try `absotui --update`/`--uninstall` in a terminal instead".to_string();
            error!("[update_uninstall] {action:?} timed out");
            let _ = tx.send(ProgressEvent::Finished(Err(message)));
        }
    }
}

// `-k` forces sudo to actually check the password just typed instead of silently
// passing on an unrelated cached ticket; `-v` only validates/extends the timestamp,
// it doesn't run anything. Once this succeeds, hello_absotui.sh's own `sudo` calls
// ride the cached ticket with no further prompt, as long as the whole run stays
// within sudo's timeout window (it does - the run itself is capped at 180s below).
async fn authenticate_sudo(password: &str) -> std::io::Result<bool> {
    let mut child = Command::new("sudo")
        .args(["-S", "-k", "-v"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(format!("{password}\n").as_bytes()).await?;
    }

    let status = child.wait().await?;
    Ok(status.success())
}

async fn run_script(action: Action, tx: &UnboundedSender<ProgressEvent>) -> Result<(), String> {
    let command_word = match action {
        Action::Update => "update",
        Action::Uninstall => "uninstall",
    };

    // Same one-liner clap.rs's run_hello_absotui builds (fresh copy of the script from
    // `stable`, run it, clean up the tmpfile) - `2>&1` merges stderr into the same
    // ordered stream we read below.
    let script = format!(
        r#"tmpfile=$(mktemp) && curl -LsSf https://github.com/pdwaldrop/Absotui/raw/stable/hello_absotui.sh -o "$tmpfile" && bash "$tmpfile" {command_word} 2>&1; rc=$?; rm -f "$tmpfile"; exit $rc"#
    );

    let mut child = Command::new("bash")
        .arg("-c")
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch installer: {e}"))?;

    // Answers the script's own interactive prompts in order: "update to latest? (Y/n)"
    // then "which install method?" for update (always "1" - download the precompiled
    // binary; see the plan's scope decision on why source-method updates stay a manual
    // `absotui --update` thing), or just "uninstall? (Y/n)" for uninstall (we already
    // got real confirmation in our own UI before getting here). Closing stdin right
    // after (dropping `stdin` at the end of this block) means any *unexpected* further
    // prompt fails fast on EOF instead of hanging until the timeout below.
    if let Some(mut stdin) = child.stdin.take() {
        let answers = match action {
            Action::Update => "y\n1\n",
            Action::Uninstall => "y\n",
        };
        let _ = stdin.write_all(answers.as_bytes()).await;
    }

    if let Some(stdout) = child.stdout.take() {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let cleaned = strip_ansi(&line);
            if !cleaned.trim().is_empty() {
                let _ = tx.send(ProgressEvent::Line(cleaned));
            }
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Installer process error: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        let code = status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        Err(format!("Installer exited with status {code}"))
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
