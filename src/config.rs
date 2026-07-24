use config::{Config as ConfigLib, File};
use serde::Deserialize;
use color_eyre::eyre::{Result, Report};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub colors: Colors,
    pub player: Player,
    pub downloads: Downloads,
}

#[derive(Debug, Deserialize)]
pub struct Colors {
    pub background_color: Vec<u8>,
    pub log_background_color: Vec<u8>,
    pub header_background_color: Vec<u8>,
    pub line_header_color: Vec<u8>,
    pub list_background_color: Vec<u8>,
    pub list_background_color_alt_row: Vec<u8>,
    pub list_selected_background_color: Vec<u8>,
    pub list_selected_foreground_color: Vec<u8>,
    pub search_bar_foreground_color: Vec<u8>,
    pub login_foreground_color: Vec<u8>,
    pub player_background_color: Vec<u8>,
    #[serde(default = "default_progress_bar_color")]
    pub progress_bar_color: Vec<u8>,
}

// Steel blue - used as a safe default for users whose existing config.toml
// predates this option, so config loading doesn't break for them.
fn default_progress_bar_color() -> Vec<u8> {
    vec![70, 130, 180]
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub cvlc: String,
    pub cvlc_term: String,
    pub address: String,
    pub port: String,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    #[serde(default = "default_auto_download_count")]
    pub auto_download_count: usize,
}

impl Default for Downloads {
    fn default() -> Self {
        Downloads { auto_download_count: default_auto_download_count() }
    }
}

// Existing config.toml files predate Auto Download entirely, so the whole
// `[downloads]` section is typically missing rather than just this field -
// `load_config` falls back to this same default when the section itself is absent.
fn default_auto_download_count() -> usize {
    5
}

/// load config from `config.toml` file
pub fn load_config() -> Result<ConfigFile> {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| { 
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let config_path = config_home_path.join("absotui/config.toml");
    let config_path_str = config_path.to_str().unwrap().to_string();

    let config = ConfigLib::builder()
        .add_source(File::with_name(&config_path_str))
        .build()
        .map_err(|e| Report::new(e))?;

    let colors: Colors = config.get("colors")
        .map_err(|e| Report::new(e))?;
    let player: Player = config.get("player")
        .map_err(|e| Report::new(e))?;
    let downloads: Downloads = config.get("downloads").unwrap_or_default();

    Ok(ConfigFile { colors, player, downloads })
}

