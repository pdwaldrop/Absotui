use log::LevelFilter;
use fern::Dispatch;
use chrono::Local;
use std::fs::OpenOptions;
use std::env;
use std::path::PathBuf;

pub fn setup_logs() -> Result<(), fern::InitError> {

    let config_home_path = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from) 
        .unwrap_or_else(|_| { 
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        });

    let log_path = config_home_path.join("absotui/absotui.log");

    // Create or append into the file
    let log_file = OpenOptions::new()
        .create(true)
        
        .append(true)
        .open(log_path) // path and name
        .unwrap();

    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                    "{} [{}] - {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    message
            ))
        })
    .level(LevelFilter::Info) 
        .chain(log_file) // redirect logs to the file 
        .apply()?; 

    Ok(())
}
