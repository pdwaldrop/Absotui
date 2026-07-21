use log::LevelFilter;
use fern::Dispatch;
use chrono::Local;
use std::fs::OpenOptions;
use std::env;
use std::path::PathBuf;

pub fn setup_logs() -> Result<(), fern::InitError> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| { 
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let log_path = config_home_path.join("absotui/absotui.log");

    // `OpenOptions::create(true)` creates the file but not its parent directory - on a
    // genuinely fresh environment (built from source and run directly, or a
    // not-yet-created XDG_CONFIG_HOME override) this is the very first fallible thing
    // main() does, so a missing ~/.config/absotui/ would otherwise panic with a raw,
    // unhelpful IO error before anything else has a chance to run.
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Create or append into the file
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?; // path and name

    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                    "{} [{}] - {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    message
            ));
        })
    .level(LevelFilter::Info) 
        .chain(log_file) // redirect logs to the file 
        .apply()?; 

    Ok(())
}
