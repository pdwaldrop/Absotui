use crate::player::vlc::start_vlc::start_vlc;
use crate::player::vlc::fetch_vlc_data::{fetch_vlc_data, fetch_vlc_is_playing};
use crate::utils::pop_up_message::clear_message;
use crate::api::me::update_media_progress::{update_media_progress_pod, update_media_progress2_pod};
use std::io::stdout;
use log::{info, error};
use crate::db::crud::{insert_listening_session, update_is_vlc_running, update_current_time, update_chapter, update_is_finished, update_is_loop_break, get_listening_session};
use crate::db::database_struct::DownloadedItem;
use crate::utils::vlc_tcp_stream::vlc_tcp_stream;

/// Plays a downloaded podcast episode with no server session at all - used when
/// `post_start_playback_session_pod` fails (server unreachable) but a local copy of the
/// episode exists (see src/utils/download_cache.rs). Shared by both handle_l_pod_home
/// and handle_l_pod, since the offline case is identical either way - no
/// `sync_session`/`update_media_progress`/`close_session` calls happen during playback
/// (no server-issued session id to attach them to), and - unlike the online path -
/// there's no Podcast Autoplay here: advancing to a next episode needs a live fetch of
/// the current New & Unfinished/episode-list queue, which isn't available offline.
/// Mirrors handle_l_book.rs's handle_l_book_offline exactly, adapted for the extra
/// podcast/episode id split and the podcast progress-update endpoints.
pub async fn handle_pod_episode_offline(
    podcast_id: String,
    episode_id: String,
    downloaded: DownloadedItem,
    port: String,
    address_player: String,
    program: String,
    username: String,
    token: String,
    server_address: String,
) {
    // Not a server-issued id - used purely as this local session's sqlite key/log tag.
    let id_session = format!("offline-{episode_id}");

    // Resume from wherever local playback last left off, if the last real
    // `listening_session` row happens to be this same episode - the download itself
    // doesn't track a position.
    let mut current_time: u32 = get_listening_session().ok().flatten()
        .filter(|s| s.id_pod == episode_id)
        .map(|s| s.current_time)
        .unwrap_or(0);

    let _ = insert_listening_session(
        id_session.clone(),
        podcast_id.clone(),
        current_time,
        downloaded.duration.clone(),
        episode_id.clone(),
        0,
        downloaded.title.clone(),
        String::new(), // author not shown for podcasts
        true,
        String::new(),
        String::new(),
    );

    let port_clone = port.clone();
    let address_player_clone = address_player.clone();
    let username_clone = username.clone();
    let podcast_id_clone = podcast_id.clone();
    let local_file_path = downloaded.file_path.clone();
    let title = downloaded.title.clone();
    let current_time_str = current_time.to_string();

    tokio::spawn(async move {
        info!("[handle_pod_episode_offline][start_vlc] VLC launched against local file (offline)");
        // content_url/token/server_address are unused whenever local_file_path is
        // Some - see start_vlc's `source` resolution.
        if let Err(e) = start_vlc(
            &current_time_str,
            &port_clone,
            address_player_clone,
            &String::new(),
            None,
            title.clone(),
            title,
            String::new(),
            String::new(),
            program,
            username_clone,
            podcast_id_clone,
            Some(local_file_path),
        ).await {
            error!("[handle_pod_episode_offline][start_vlc] {e}");
        }
    });

    let mut stdout = stdout();
    let _ = clear_message(&mut stdout, 3);

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let _ = update_is_vlc_running("1", username.as_str());

    loop {
        match fetch_vlc_data(port.clone(), address_player.clone()).await {
            Ok(Some(data_fetched_from_vlc)) => {
                let _ = update_current_time(data_fetched_from_vlc, id_session.as_str());
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                match vlc_tcp_stream(address_player.as_str(), port.as_str(), "chapter") {
                    Ok(response) => {
                        let _ = update_chapter(response.as_str(), id_session.as_str());
                    }
                    Err(e) => info!("[handle_pod_episode_offline] Error: {e}"),
                }

                match fetch_vlc_is_playing(port.clone(), address_player.clone()).await {
                    Ok(true) => {
                        current_time = data_fetched_from_vlc;
                    }
                    Ok(false) => {
                        info!("[handle_pod_episode_offline][Finished] Track finished");
                        let _ = update_is_finished("1", id_session.as_str());
                        // Best-effort only - ignored whether the server is back or not.
                        let _ = update_media_progress2_pod(&podcast_id, Some(&token), Some(data_fetched_from_vlc), &downloaded.duration, true, &episode_id, server_address.clone()).await;
                        let _ = update_is_loop_break("1", username.as_str());
                        let _ = update_is_vlc_running("0", username.as_str());
                        break;
                    }
                    Err(_) => {
                        let _ = update_is_vlc_running("0", username.as_str());
                        info!("[handle_pod_episode_offline][Quit] Item {episode_id} closed at {data_fetched_from_vlc}s");
                        let _ = update_media_progress_pod(&podcast_id, Some(&token), Some(data_fetched_from_vlc), &downloaded.duration, &episode_id, server_address.clone()).await;
                        let _ = update_is_loop_break("1", username.as_str());
                        break;
                    }
                }
            }
            Ok(None) => {
                let _ = update_is_vlc_running("0", username.as_str());
                info!("[handle_pod_episode_offline][None] Item {episode_id} closed at {current_time}s");
                let _ = update_media_progress_pod(&podcast_id, Some(&token), Some(current_time), &downloaded.duration, &episode_id, server_address.clone()).await;
                let _ = update_is_loop_break("1", username.as_str());
                break;
            }
            Err(e) => {
                error!("[handle_pod_episode_offline][Err(e)]{e}");
                let _ = update_is_vlc_running("0", username.as_str());
                let _ = update_is_loop_break("1", username.as_str());
                break;
            }
        }
    }
}
