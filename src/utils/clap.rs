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
        std::process::Command::new("sh")
            .arg("-c")
            .arg(
                r#"bash -c 'expected_sha256="b3a36a51b97652040c5a95aac4690df2f5bfc1197e9fc32694087301a942f8bc" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/pdwaldrop/Absotui/raw/stable/hello_absotui.sh -o "$tmpfile" && bash "$tmpfile" uninstall && rm -f "$tmpfile"'"#,
            )
            .status()
            .expect("failed to run uninstall script");
        std::process::exit(0);
    }
    if matches.get_flag("update") {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(
                r#"bash -c 'expected_sha256="b3a36a51b97652040c5a95aac4690df2f5bfc1197e9fc32694087301a942f8bc" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/pdwaldrop/Absotui/raw/stable/hello_absotui.sh -o "$tmpfile" && bash "$tmpfile" update && rm -f "$tmpfile"'"#,
            )
            .status()
            .expect("failed to run update script");
        std::process::exit(0);
    }

}

