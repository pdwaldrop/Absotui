use std::env;
use std::path::PathBuf;
use color_eyre::eyre::Result;
use crate::api::library_items::get_cover::get_cover;

fn covers_dir() -> PathBuf {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    config_home_path.join("absotui/covers")
}

/// Path a given item's cached cover would live at, regardless of whether it's been
/// fetched yet - image format is auto-detected from the file's contents at load time,
/// so no extension is needed.
pub fn cover_cache_path(item_id: &str) -> PathBuf {
    covers_dir().join(item_id)
}

/// Fetches an item's cover and writes it to the local cache, if not already cached.
/// Meant to be run in a background task - rendering just polls `cover_cache_path` for
/// the file's existence rather than waiting on this directly.
pub async fn fetch_and_cache_cover(token: String, item_id: String, server_address: String) -> Result<()> {
    let path = cover_cache_path(&item_id);
    if path.exists() {
        return Ok(());
    }

    let bytes = get_cover(&token, &item_id, &server_address).await?;

    std::fs::create_dir_all(covers_dir())?;
    std::fs::write(path, bytes)?;

    Ok(())
}
