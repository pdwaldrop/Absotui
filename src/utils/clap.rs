use clap::{Arg, Command};

pub fn clap() {
    let matches = Command::new("absotui")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("update")
                .long("update")
                .help("Run update script via curl")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("uninstall")
                .long("uninstall")
                .help("Run uninstall script via curl")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("uninstall") {
        run_hello_absotui("uninstall");
    }
    if matches.get_flag("update") {
        run_hello_absotui("update");
    }
}

// Downloads hello_absotui.sh fresh from the stable branch and runs it - no checksum is
// hardcoded here. An earlier version of this function embedded one directly in the
// binary, which inevitably went stale the next time the script changed (as it just
// did) and broke update/uninstall for anyone still on an older binary - exactly the
// class of bug the script's own checksum handling was already supposed to have
// eliminated. hello_absotui.sh verifies itself against the latest release's actual
// published checksum at run time instead (see fetch_expected_checksum there), so
// there's nothing left for this binary to keep in sync.
fn run_hello_absotui(command: &str) {
    let script = format!(
        r#"tmpfile=$(mktemp) && curl -LsSf https://github.com/pdwaldrop/Absotui/raw/stable/hello_absotui.sh -o "$tmpfile" && bash "$tmpfile" {command} && rm -f "$tmpfile""#
    );
    let status = std::process::Command::new("bash")
        .arg("-c")
        .arg(script)
        .status()
        .expect("failed to run hello_absotui.sh");
    std::process::exit(status.code().unwrap_or(1));
}

