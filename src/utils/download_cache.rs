use std::env;
use std::path::PathBuf;
use color_eyre::eyre::Result;
use log::{info, error};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use crate::api::library_items::play_lib_item_or_pod::post_start_playback_session_book;
use crate::api::sessions::close_open_session::close_session_without_send_prg_data;
use crate::db::crud::{insert_download, delete_download, get_download, list_downloaded_ids};

fn downloads_dir() -> PathBuf {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    config_home_path.join("absotui/downloads")
}

/// Same path-traversal reasoning as `cover_cache::cover_cache_path` - `item_id` comes
/// straight from the configured server, unvalidated, so both it and `ext` (also
/// server-derived, via the response's `Content-Type`) are stripped down to characters
/// that can never form `/`, `\`, or `..` before being used to build a path.
pub fn download_audio_path(item_id: &str, ext: &str) -> PathBuf {
    let safe_id: String = item_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    let safe_ext: String = ext.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    downloads_dir().join(format!("{safe_id}.{safe_ext}"))
}

/// Real Audiobookshelf audio files are one of a handful of known containers - anything
/// else (or a missing/malformed header) falls back to `m4b`, the most common audiobook
/// format, rather than leaving the file extensionless (VLC's local-file demuxer
/// selection leans on the extension more than it does for network streams).
fn extension_from_content_type(content_type: Option<&str>) -> &'static str {
    match content_type.unwrap_or_default() {
        "audio/mp4" | "audio/x-m4b" | "audio/x-m4a" => "m4b",
        "audio/mpeg" => "mp3",
        "audio/ogg" => "ogg",
        "audio/flac" | "audio/x-flac" => "flac",
        "audio/aac" => "aac",
        "audio/wav" | "audio/x-wav" => "wav",
        _ => "m4b",
    }
}

pub fn is_downloaded(username: &str, item_id: &str) -> bool {
    get_download(username, item_id).is_some_and(|d| std::path::Path::new(&d.file_path).exists())
}

/// Downloads a book's audio file for offline playback, if not already downloaded.
/// Meant to be run in a background task, same as `cover_cache::fetch_and_cache_cover`.
///
/// There's no lighter-weight endpoint to resolve a book's direct-play URL than
/// actually starting a playback session (that's how `handle_l_book` itself gets
/// `content_url`), so this opens one purely to read `content_url`/`duration` off the
/// response and immediately closes it again - the server never sees a real listen out
/// of this.
pub async fn download_book(token: String, item_id: String, title: String, author: String, username: String, server_address: String) -> Result<()> {
    if is_downloaded(&username, &item_id) {
        return Ok(());
    }

    let info_item = post_start_playback_session_book(Some(&token), &item_id, server_address.clone()).await?;
    // info_item: [current_time, content_url, duration, id_session, title, subtitle, author, chapters_json]
    let content_url = &info_item[1];
    let duration = &info_item[2];
    let id_session = &info_item[3];

    if let Err(e) = close_session_without_send_prg_data(Some(&token), id_session, server_address.clone()).await {
        error!("[download_book] failed to close transient session for {item_id}: {e}");
    }

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{server_address}{content_url}"))
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?;

    let ext = extension_from_content_type(response.headers().get(CONTENT_TYPE).and_then(|v| v.to_str().ok()));
    let bytes = response.bytes().await?;

    std::fs::create_dir_all(downloads_dir())?;
    let path = download_audio_path(&item_id, ext);
    std::fs::write(&path, &bytes)?;

    insert_download(&username, &item_id, &path.to_string_lossy(), duration, &title, &author)?;
    info!("[download_book] downloaded {item_id} ({} bytes) to {path:?}", bytes.len());

    Ok(())
}

/// Removes a book's local download and its db row, if any.
pub fn remove_download(username: &str, item_id: &str) -> Result<()> {
    if let Some(downloaded) = get_download(username, item_id) {
        let _ = std::fs::remove_file(&downloaded.file_path);
    }
    delete_download(username, item_id)?;
    Ok(())
}

/// Settings > Auto Download: keeps the local download set mirroring the `count` most
/// recently played books in Continue Listening (`ids` is already server-ordered
/// most-recent-first) - downloads any of them not already downloaded, and removes any
/// existing download whose id has fallen out of that top-`count` window (finished,
/// pushed out by newer activity, or the window shrank), so disk usage stays bounded
/// rather than growing forever. Runs in the background and is meant to be called once
/// per Continue Listening refresh (see `App::new()`) - downloads happen one at a time
/// rather than in parallel, so this doesn't try to saturate the connection with
/// several hundred-MB-plus files at once.
pub fn sync_auto_downloads(username: String, token: String, server_address: String, ids: Vec<String>, titles: Vec<String>, authors: Vec<String>, count: usize) {
    tokio::spawn(async move {
        let ids: Vec<String> = ids.into_iter().take(count).collect();

        if let Ok(existing) = list_downloaded_ids(&username) {
            for stale_id in existing.into_iter().filter(|id| !ids.contains(id)) {
                if let Err(e) = remove_download(&username, &stale_id) {
                    error!("[sync_auto_downloads] failed to prune {stale_id}: {e}");
                }
            }
        }

        for (i, id) in ids.iter().enumerate() {
            if is_downloaded(&username, id) {
                continue;
            }
            let title = titles.get(i).cloned().unwrap_or_default();
            let author = authors.get(i).cloned().unwrap_or_default();
            if let Err(e) = download_book(token.clone(), id.clone(), title, author, username.clone(), server_address.clone()).await {
                error!("[sync_auto_downloads] {id}: {e}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legitimate_uuid_style_id_is_unchanged() {
        let id = "a1b2c3d4-e5f6-4789-a0bc-def012345678";
        assert_eq!(download_audio_path(id, "m4b").file_name().unwrap().to_str().unwrap(), format!("{id}.m4b"));
    }

    #[test]
    fn path_traversal_id_cannot_escape_downloads_dir() {
        let malicious = "../../../../home/user/.ssh/authorized_keys";
        let path = download_audio_path(malicious, "m4b");
        assert!(path.starts_with(downloads_dir()));
        assert_eq!(path.parent().unwrap(), downloads_dir());
    }

    #[test]
    fn absolute_path_id_cannot_escape_downloads_dir() {
        let malicious = "/home/user/.ssh/authorized_keys";
        let path = download_audio_path(malicious, "m4b");
        assert!(path.starts_with(downloads_dir()));
        assert_eq!(path.parent().unwrap(), downloads_dir());
    }

    #[test]
    fn malicious_ext_cannot_introduce_a_path_separator() {
        let path = download_audio_path("real-id", "../../../etc/passwd");
        assert!(path.starts_with(downloads_dir()));
        assert_eq!(path.parent().unwrap(), downloads_dir());
    }
}
