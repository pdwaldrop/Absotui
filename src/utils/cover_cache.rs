use std::env;
use std::io::Cursor;
use std::path::PathBuf;
use color_eyre::eyre::Result;
use crate::api::library_items::get_cover::get_cover;
use crate::api::library_items::get_audio_file_range::get_audio_file_prefix;

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

/// Fetches a podcast episode's own embedded cover art (from its MP3 ID3 tag) and writes
/// it to the local cache, if not already cached. Leaves nothing behind - and therefore
/// falls back to the parent podcast's cover on every subsequent render - if the file
/// doesn't actually carry a picture frame within the fetched prefix; callers only invoke
/// this for episodes flagged at scan time as having one, so that should be rare.
pub async fn fetch_and_cache_episode_cover(token: String, episode_id: String, library_item_id: String, ino: String, server_address: String) -> Result<()> {
    let path = cover_cache_path(&episode_id);
    if path.exists() {
        return Ok(());
    }

    let prefix = get_audio_file_prefix(&token, &library_item_id, &ino, &server_address).await?;

    let tag = match id3::Tag::read_from2(Cursor::new(prefix)) {
        Ok(tag) => tag,
        Err(e) => {
            log::warn!("[fetch_and_cache_episode_cover] episode {episode_id}: no readable ID3 tag in fetched prefix: {e}");
            return Ok(());
        }
    };

    let Some(picture) = tag.pictures().next() else {
        log::warn!("[fetch_and_cache_episode_cover] episode {episode_id}: ID3 tag has no picture frame despite embeddedCoverArt flag");
        return Ok(());
    };

    std::fs::create_dir_all(covers_dir())?;
    std::fs::write(path, &picture.data)?;

    Ok(())
}
