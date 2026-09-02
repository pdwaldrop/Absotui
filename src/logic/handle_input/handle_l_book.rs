use crate::player::vlc::start_vlc::start_vlc;
use crate::player::vlc::fetch_vlc_data::{fetch_vlc_data, fetch_vlc_is_playing};
use crate::player::vlc::exec_nc::exec_nc;
use crate::utils::pop_up_message::{clear_message, NEEDS_TERMINAL_CLEAR};
use std::sync::atomic::Ordering;
use crate::api::me::update_media_progress::{update_media_progress_book, update_media_progress2_book};
use crate::api::library_items::play_lib_item_or_pod::{post_start_playback_session_book, AudioTrack, find_track_index};
use crate::api::sessions::sync_open_session::sync_session;
use crate::api::sessions::close_open_session::close_session_without_send_prg_data;
use std::io::stdout;
use log::{info, error};
use crate::db::crud::{insert_listening_session, update_is_vlc_running, update_current_time, get_speed_rate, update_chapter, update_elapsed_time, update_is_finished, update_is_loop_break, get_download, get_listening_session, update_pending_seek, delete_user, update_login_err};
use crate::api::server::refresh_token::{maybe_refresh_token, RefreshOutcome};
use crate::db::database_struct::DownloadedItem;
use crate::utils::vlc_tcp_stream::vlc_tcp_stream;
use crate::player::vlc::quit_vlc::{pkill_vlc, quit_vlc};
use crate::player::integrated::handle_key_player::seek_to_absolute_time;
use crate::utils::convert_seconds::progress_time_diff;

/// Launches VLC for one track and returns immediately, without waiting for it to
/// finish. `start_vlc` shells out via `Command::output()`, which blocks until the
/// child process itself exits - ie. for the length of the whole track - so every call
/// site (the very first launch, and every later track switch/advance) must fire this
/// from its own `tokio::spawn` rather than `.await`ing `start_vlc` inline, or whichever
/// loop called it would hang until VLC quits instead of continuing to poll it.
#[allow(clippy::too_many_arguments)]
fn spawn_track_vlc(
    log_ctx: &'static str,
    relative_start: u32,
    port: String,
    address_player: String,
    content_url: String,
    token: Option<String>,
    title: String,
    subtitle: String,
    author: String,
    server_address: String,
    program: String,
    is_cvlc: String,
    username: String,
    id_item: String,
    local_file_path: Option<String>,
) {
    tokio::spawn(async move {
        // this info! is not the most reliable to know is VLC is really launched
        info!("[{log_ctx}][start_vlc] VLC successfully launched");
        if let Err(e) = start_vlc(
            &relative_start.to_string(),
            &port,
            address_player,
            &content_url,
            token.as_ref(),
            title,
            subtitle,
            author,
            server_address,
            program,
            is_cvlc,
            username,
            id_item,
            local_file_path,
        ).await {
            error!("[{log_ctx}][start_vlc] {e}");
        }
    });
}

pub async fn handle_l_book(
    token: Option<&String>,
    refresh_token: Option<&String>,
    ids_library_items: Vec<String>,
    selected: Option<usize>,
    port: String,
    address_player: String,
    server_address: String,
    program: String,
    is_cvlc: String,
    is_cvlc_term: String,
    username: String,
) {

    pkill_vlc();

    if let Some(index) = selected
        && let Some(id) = ids_library_items.get(index)
            && let Some(token) = token {
                // Owned and mutable so the proactive refresh check below (run once per
                // loop iteration) can reassign them in place - this task never touches
                // the live `App`, so keeping its own token fresh is entirely on it (see
                // CLAUDE.md's "one owner" note).
                let mut token = token.clone();
                let mut refresh_token = refresh_token.cloned().unwrap_or_default();

                // Checked up front so both branches below (online, and the offline
                // fallback if the session-start call fails) know whether a local copy
                // exists - see src/utils/download_cache.rs.
                let downloaded = get_download(&username, id);

                match post_start_playback_session_book(Some(&token), id, server_address.clone()).await {
                    Err(e) => {
                        if let Some(downloaded) = downloaded {
                            info!("[handle_l_book] Couldn't start an online playback session ({e}) - falling back to the downloaded copy of {id}");
                            handle_l_book_offline(
                                id.clone(),
                                downloaded,
                                port,
                                address_player,
                                program,
                                is_cvlc,
                                username,
                                token.clone(),
                                refresh_token.clone(),
                                server_address,
                            ).await;
                        } else {
                            error!("[handle_l_book] Failed to start playback session: {e}");
                            eprintln!("Failed to start playback session");
                            // Without this, wait_prev_session_finished's poll loop (blocking
                            // every future play attempt until this flips back to "1") never
                            // sees it happen - a single transient failure here (network blip,
                            // server 5xx) would otherwise permanently wedge playback until the
                            // app is quit cleanly with `Q`.
                            let _ = update_is_loop_break("1", username.as_str());
                        }
                    }
                    Ok(info_item) => {

                    let mut current_time: u32 = info_item[0].parse::<f64>().unwrap().round() as u32;

                    info!("[handle_l_book][post_start_playback_session_book] OK");
                    info!("[handle_l_book][post_start_playback_session_book] Item {id} started at {current_time}s");

                    // A book split across separate per-chapter files (rather than a single
                    // .m4b) gets one AudioTrack per file here - see find_track_index's doc
                    // comment. A single-file book is simply the 1-track case and every
                    // "is there a next track" check below naturally becomes a no-op.
                    let tracks: Vec<AudioTrack> = serde_json::from_str(&info_item[8]).unwrap_or_default();
                    let mut current_track_idx = if tracks.is_empty() { 0 } else { find_track_index(&tracks, f64::from(current_time)) };
                    let mut track_base_offset: u32 = tracks.get(current_track_idx).map(|t| t.start_offset.round() as u32).unwrap_or(0);

                    // If this book was downloaded for offline playback, prefer the local
                    // copy of whichever track we're about to play even though the server is
                    // reachable (existing "prefer-local playback" behavior) - matched by
                    // track index, since a legacy single-file download only covers track 0.
                    let local_tracks: Vec<String> = downloaded.as_ref()
                        .map(|d| d.tracks.iter().map(|t| t.local_path.clone()).collect())
                        .unwrap_or_default();


                    let _ = insert_listening_session(
                        info_item[3].clone(), // id_session
                        id.clone(), // id_item
                        current_time,
                        info_item[2].clone(), // total item duration
                        String::new(), // empty here, because it's for podcasts
                        0, // elapsed time start at 0 seconds
                        info_item[4].clone(), // title
                        info_item[6].clone(), // author
                        true, // is_playback
                        String::new(), // chapter
                        info_item[7].clone(), // chapters (JSON array of {id, title, start, end})
                        );

                    // Relative to the starting track's own beginning, not the book-wide
                    // resume position - VLC only ever has one file loaded at a time.
                    let initial_relative_start = current_time.saturating_sub(track_base_offset);
                    let initial_local_file_path = local_tracks.get(current_track_idx).cloned();

                    spawn_track_vlc(
                        "handle_l_book",
                        initial_relative_start,
                        port.clone(),
                        address_player.clone(),
                        info_item[1].clone(), // content url (starting track)
                        Some(token.clone()),
                        info_item[4].clone(), // title
                        info_item[5].clone(), // subtitle
                        info_item[6].clone(), // author
                        server_address.clone(),
                        program.clone(),
                        is_cvlc.clone(),
                        username.clone(),
                        id.clone(),
                        initial_local_file_path,
                    );


                    if is_cvlc_term == "1" {
                        let port_clone = port.clone();
                        let address_player_clone = address_player.clone();
                        tokio::spawn(async move {
                            if let Err(e) = exec_nc(&port_clone, address_player_clone).await {
                                error!("[handle_l_book][exec_nc] {e}");
                            }
                        });
                    }



                    // clear loading message (from app.rs) when vlc is launched
                    let mut stdout = stdout();
                    let _ = clear_message(&mut stdout, 3);
                    NEEDS_TERMINAL_CLEAR.store(true, Ordering::Relaxed);


                    // Gives VLC time to accept the RC connection before polling it.
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                    // Seeded to a value fetch_vlc_data's first real reading is very unlikely
                    // to land on exactly, so the first poll iteration doesn't get misread as
                    // "paused" by the data_fetched_from_vlc == last_current_time check below.
                    let mut last_current_time: u32 = 3;
                    let mut progress_sync: u32 = 3;

                    let _ = update_is_vlc_running("1", username.as_str());

                    let mut trigger = 1;

                    // Stays false until fetch_vlc_data first returns a real position -
                    // guards the Ok(None)/Err(e) cleanup arms below from syncing
                    // `current_time` (still whatever it was at session start) when VLC
                    // was launched and quit before ever reporting anything real: for a
                    // book with no prior progress that's 0, and re-sending it as the
                    // "closed at" position reads as progress being wiped rather than
                    // "nothing was actually observed to sync."
                    let mut got_real_data = false;

                    loop {
                        // Keeps this task's own copy of the token fresh independently of
                        // the main App (see handle_l_book's module-level context in
                        // CLAUDE.md's "one owner" note) - a book can play for hours,
                        // easily outliving Audiobookshelf's ~1 hour default access token.
                        if maybe_refresh_token(&mut token, &mut refresh_token, username.as_str(), server_address.as_str()).await == RefreshOutcome::Failed {
                            let _ = delete_user(username.as_str());
                            let _ = update_login_err("Your session expired - restart Absotui to log in again");
                            let _ = close_session_without_send_prg_data(Some(&token), &info_item[3], server_address.clone()).await;
                            let _ = quit_vlc(&address_player, &port);
                            pkill_vlc();
                            let _ = update_is_vlc_running("0", username.as_str());
                            let _ = update_is_loop_break("1", username.as_str());
                            break;
                        }

                        // A chapter jump ("c" list, or P/U) from the UI - only this loop
                        // knows the real current_track_idx/track_base_offset, so it's the
                        // sole place that decides whether fulfilling it is a same-file seek
                        // or requires swapping VLC to a different file. See
                        // update_pending_seek's doc comment.
                        if let Ok(Some(session)) = get_listening_session()
                            && !session.pending_seek.is_empty()
                            && let Ok(target) = session.pending_seek.parse::<f64>() {
                                let _ = update_pending_seek("", info_item[3].as_str());
                                let target_track_idx = if tracks.is_empty() { 0 } else { find_track_index(&tracks, target) };
                                let target_track_base = tracks.get(target_track_idx).map(|t| t.start_offset.round() as u32).unwrap_or(0);
                                let relative_target = (target.round() as u32).saturating_sub(target_track_base);

                                if target_track_idx == current_track_idx {
                                    let _ = seek_to_absolute_time(&address_player, &port, relative_target);
                                } else {
                                    let _ = quit_vlc(&address_player, &port);
                                    pkill_vlc();
                                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                    current_track_idx = target_track_idx;
                                    track_base_offset = target_track_base;
                                    let content_url = tracks.get(current_track_idx).map(|t| t.content_url.clone()).unwrap_or_default();
                                    let local_file_path = local_tracks.get(current_track_idx).cloned();

                                    spawn_track_vlc(
                                        "handle_l_book",
                                        relative_target,
                                        port.clone(),
                                        address_player.clone(),
                                        content_url,
                                        Some(token.clone()),
                                        info_item[4].clone(),
                                        info_item[5].clone(),
                                        info_item[6].clone(),
                                        server_address.clone(),
                                        program.clone(),
                                        is_cvlc.clone(),
                                        username.clone(),
                                        id.clone(),
                                        local_file_path,
                                    );
                                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                    last_current_time = 3;
                                    progress_sync = 3;
                                    trigger = 1;
                                }
                        }

                        match fetch_vlc_data(port.clone(), address_player.clone()).await {
                            Ok(Some(data_fetched_from_vlc)) => {
                                got_real_data = true;
                                let book_wide_time = track_base_offset.saturating_add(data_fetched_from_vlc);

                                // Book-wide, not just this file's own raw position, so Continue
                                // Listening / resume reflect the real position across a
                                // multi-file book.
                                let _ = update_current_time(book_wide_time, info_item[3].as_str());

                                // Paces this poll loop to once a second.
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                if data_fetched_from_vlc == last_current_time {
                                    progress_sync = 0; // the track is in pause
                                } else {
                                    let speed_rate_str = get_speed_rate(username.as_str());
                                    let speed_rate = speed_rate_str.parse::<f64>().unwrap_or(1.0);
                                    let current_time_adjusted = f64::from(current_time) / speed_rate;
                                    let book_wide_time_adjusted = f64::from(book_wide_time) / speed_rate;
                                    let diff = progress_time_diff(book_wide_time_adjusted, current_time_adjusted);
                                    // A diff this large means a chapter/10s jump rather than
                                    // normal playback advancing - clamp the sync amount to 1s
                                    // instead of reporting the raw (inaccurate) jump size.
                                    if diff > 20 {
                                        progress_sync += 1;
                                    } else {
                                    progress_sync = diff;
                                    }
                                }
                                last_current_time = data_fetched_from_vlc;

                                match vlc_tcp_stream(address_player.as_str(), port.as_str(), "chapter") {
                                    Ok(response) => {
                                       let _ = update_chapter(response.as_str(), info_item[3].as_str());
                                    }
                                    Err(e) => info!("Error: {e}"),
                                }


                                match fetch_vlc_is_playing(port.clone(), address_player.clone()).await {
                                    Ok(true) => {
                                        // to sync progress in the server each 10 seconds
                                        if trigger == 10 {
                                                let _ = sync_session(Some(&token), &info_item[3],Some(book_wide_time), progress_sync, server_address.clone()).await;
                                                let _ = update_media_progress_book(id, Some(&token), Some(book_wide_time), &info_item[2], server_address.clone()).await;

                                                let _ = update_elapsed_time(progress_sync, info_item[3].as_str());

                                                current_time = book_wide_time;
                                                progress_sync = 0;
                                                trigger = 0;

                                        } else if progress_sync != 0 {
                                            trigger += 1;
                                        } else if progress_sync == 0 {
                                            trigger += 0;
                                        }
                                    },
                                    // The track ended naturally but VLC is still open - distinct
                                    // from Err below, where the user closed VLC itself.
                                    Ok(false) => {
                                        if current_track_idx + 1 < tracks.len() {
                                            // End of this file, not the end of the book -
                                            // advance to the next one within the same
                                            // session instead of closing/marking finished.
                                            info!("[handle_l_book][Finished] Track {current_track_idx} finished, advancing to track {}", current_track_idx + 1);
                                            let _ = update_current_time(book_wide_time, info_item[3].as_str());

                                            let _ = quit_vlc(&address_player, &port);
                                            pkill_vlc();
                                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                            current_track_idx += 1;
                                            track_base_offset = tracks[current_track_idx].start_offset.round() as u32;
                                            let content_url = tracks[current_track_idx].content_url.clone();
                                            let local_file_path = local_tracks.get(current_track_idx).cloned();

                                            spawn_track_vlc(
                                                "handle_l_book",
                                                0,
                                                port.clone(),
                                                address_player.clone(),
                                                content_url,
                                                Some(token.clone()),
                                                info_item[4].clone(),
                                                info_item[5].clone(),
                                                info_item[6].clone(),
                                                server_address.clone(),
                                                program.clone(),
                                                is_cvlc.clone(),
                                                username.clone(),
                                                id.clone(),
                                                local_file_path,
                                            );
                                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                            current_time = book_wide_time;
                                            last_current_time = 3;
                                            progress_sync = 3;
                                            trigger = 1;
                                            continue;
                                        }

                                        let is_finised = true;
                                        info!("[handle_l_book][Finished] Track finished");

                                        let _ = update_is_finished("1", info_item[3].as_str());

                                        let _ = close_session_without_send_prg_data(Some(&token), &info_item[3],  server_address.clone()).await;
                                        info!("[handle_l_book][Finished] Session successfully closed");
                                        let _ = update_media_progress2_book(id, Some(&token), Some(book_wide_time), &info_item[2], is_finised, server_address).await;
                                        info!("[handle_l_book][Finished] VLC stopped");
                                        info!("[handle_l_book][Finished] Item {id} closed at {book_wide_time}s");
                                        let _ = update_is_loop_break("1", username.as_str());

                                        let _ = update_is_vlc_running("0", username.as_str());
                                        break;
                                    },
                                    // fetch_vlc_is_playing errors when VLC itself is no longer
                                    // running - the user closed it, rather than the track ending.
                                    Err(_) => {
                                        let _ = update_is_vlc_running("0", username.as_str());
                                        info!("[handle_l_book][Quit]");
                                        let _ = close_session_without_send_prg_data(Some(&token), &info_item[3],  server_address.clone()).await;
                                        info!("[handle_l_book][Quit] Session successfully closed");
                                        let _ = update_media_progress_book(id, Some(&token), Some(book_wide_time), &info_item[2], server_address).await;
                                        info!("[handle_l_book][Quit] VLC closed");
                                        info!("[handle_l_book][Quit] Item {id} closed at {book_wide_time}s");
                                        let _ = update_is_loop_break("1", username.as_str());
                                        break;
                                    }
                                }

                            }
                            // VLC was launched and quit before ever reporting a position.
                            Ok(None) => {
                                let _ = update_is_vlc_running("0", username.as_str());
                                info!("[handle_l_book][None]");
                                let _ = close_session_without_send_prg_data(Some(&token), &info_item[3],  server_address.clone()).await;
                                info!("[handle_l_book][None] Session successfully closed");
                                if got_real_data {
                                    let _ = update_media_progress_book(id, Some(&token), Some(current_time), &info_item[2], server_address.clone()).await;
                                    info!("[handle_l_book][None] VLC closed");
                                    info!("[handle_l_book][None] Item {id} closed at {current_time}s");
                                } else {
                                    info!("[handle_l_book][None] VLC closed - no real playback data was ever fetched, skipping progress sync");
                                }

                                let _ = update_is_loop_break("1", username.as_str());
                                break;
                            }
                            Err(e) => {
                                error!("[handle_l_book][Err(e)]{e}");
                                let _ = update_is_vlc_running("0", username.as_str());
                                let _ = close_session_without_send_prg_data(Some(&token), &info_item[3],  server_address.clone()).await;
                                if got_real_data {
                                    let _ = update_media_progress_book(id, Some(&token), Some(current_time), &info_item[2], server_address.clone()).await;
                                }
                                let _ = update_is_loop_break("1", username.as_str());
                                break;
                            }
                        }
                    }
                    }
                }
            }
}

/// Plays a downloaded book with no server session at all - used when
/// `post_start_playback_session_book` fails (server unreachable) but a local copy of
/// the book exists (see src/utils/download_cache.rs). No `sync_session` /
/// `update_media_progress` / `close_session` calls happen during playback, since there
/// is no server-issued session id to attach them to - the local `listening_session`
/// row is kept current for local resume, and a single best-effort progress push (its
/// result ignored either way) happens when playback stops, in case connectivity
/// returned by then. Deliberately doesn't retry or queue that push - a fuller
/// offline-sync subsystem is out of scope for this first pass.
///
/// Sequences through `downloaded.tracks` exactly like the online loop above sequences
/// through the server's `AudioTrack`s (same book-wide-time accounting, same
/// pending_seek handling for chapter jumps) - a book downloaded as several per-chapter
/// files plays and navigates the same offline as online, not just its first file.
async fn handle_l_book_offline(
    id: String,
    downloaded: DownloadedItem,
    port: String,
    address_player: String,
    program: String,
    is_cvlc: String,
    username: String,
    mut token: String,
    mut refresh_token: String,
    server_address: String,
) {
    // Not a server-issued id - used purely as this local session's sqlite key/log tag.
    let id_session = format!("offline-{id}");

    // Resume from wherever local playback last left off, if the last real
    // `listening_session` row happens to be this same book - the download itself
    // doesn't track a position.
    let mut current_time: u32 = get_listening_session().ok().flatten()
        .filter(|s| s.id_item == id)
        .map(|s| s.current_time)
        .unwrap_or(0);

    let _ = insert_listening_session(
        id_session.clone(),
        id.clone(),
        current_time,
        downloaded.duration.clone(),
        String::new(),
        0,
        downloaded.title.clone(),
        downloaded.author.clone(),
        true,
        String::new(),
        downloaded.chapters.clone(),
    );

    let mut current_track_idx = if downloaded.tracks.is_empty() { 0 } else {
        downloaded.tracks.iter()
            .position(|t| f64::from(current_time) < t.start_offset + t.duration)
            .unwrap_or(downloaded.tracks.len().saturating_sub(1))
    };
    let mut track_base_offset: u32 = downloaded.tracks.get(current_track_idx).map(|t| t.start_offset.round() as u32).unwrap_or(0);

    let title = downloaded.title.clone();
    let author = downloaded.author.clone();

    let initial_relative_start = current_time.saturating_sub(track_base_offset);
    let initial_local_file_path = downloaded.tracks.get(current_track_idx).map(|t| t.local_path.clone());

    // content_url/token/server_address are unused whenever local_file_path is Some -
    // see start_vlc's `source` resolution.
    spawn_track_vlc(
        "handle_l_book_offline",
        initial_relative_start,
        port.clone(),
        address_player.clone(),
        String::new(),
        None,
        title.clone(),
        title.clone(),
        author.clone(),
        String::new(),
        program.clone(),
        is_cvlc.clone(),
        username.clone(),
        id.clone(),
        initial_local_file_path,
    );

    let mut stdout = stdout();
    let _ = clear_message(&mut stdout, 3);
    NEEDS_TERMINAL_CLEAR.store(true, Ordering::Relaxed);

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let _ = update_is_vlc_running("1", username.as_str());

    // See the online loop's got_real_data comment above.
    let mut got_real_data = false;

    loop {
        // Same proactive-refresh reasoning as the online loop above - this path is
        // reached because the online session-start call already failed once, so the
        // token may well already be the reason, and the best-effort progress push at
        // the end of this loop still needs a usable one.
        if maybe_refresh_token(&mut token, &mut refresh_token, username.as_str(), server_address.as_str()).await == RefreshOutcome::Failed {
            let _ = delete_user(username.as_str());
            let _ = update_login_err("Your session expired - restart Absotui to log in again");
            let _ = quit_vlc(&address_player, &port);
            pkill_vlc();
            let _ = update_is_vlc_running("0", username.as_str());
            let _ = update_is_loop_break("1", username.as_str());
            break;
        }

        // Same pending_seek mechanism as the online loop (handle_l_book) - see
        // update_pending_seek's doc comment.
        if let Ok(Some(session)) = get_listening_session()
            && !session.pending_seek.is_empty()
            && let Ok(target) = session.pending_seek.parse::<f64>() {
                let _ = update_pending_seek("", id_session.as_str());
                let target_track_idx = if downloaded.tracks.is_empty() { 0 } else {
                    downloaded.tracks.iter()
                        .position(|t| target < t.start_offset + t.duration)
                        .unwrap_or(downloaded.tracks.len().saturating_sub(1))
                };
                let target_track_base = downloaded.tracks.get(target_track_idx).map(|t| t.start_offset.round() as u32).unwrap_or(0);
                let relative_target = (target.round() as u32).saturating_sub(target_track_base);

                if target_track_idx == current_track_idx {
                    let _ = seek_to_absolute_time(&address_player, &port, relative_target);
                } else {
                    let _ = quit_vlc(&address_player, &port);
                    pkill_vlc();
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                    current_track_idx = target_track_idx;
                    track_base_offset = target_track_base;
                    let local_file_path = downloaded.tracks.get(current_track_idx).map(|t| t.local_path.clone());

                    spawn_track_vlc(
                        "handle_l_book_offline",
                        relative_target,
                        port.clone(),
                        address_player.clone(),
                        String::new(),
                        None,
                        title.clone(),
                        title.clone(),
                        author.clone(),
                        String::new(),
                        program.clone(),
                        is_cvlc.clone(),
                        username.clone(),
                        id.clone(),
                        local_file_path,
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
        }

        match fetch_vlc_data(port.clone(), address_player.clone()).await {
            Ok(Some(data_fetched_from_vlc)) => {
                got_real_data = true;
                let book_wide_time = track_base_offset.saturating_add(data_fetched_from_vlc);
                let _ = update_current_time(book_wide_time, id_session.as_str());
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                match vlc_tcp_stream(address_player.as_str(), port.as_str(), "chapter") {
                    Ok(response) => {
                        let _ = update_chapter(response.as_str(), id_session.as_str());
                    }
                    Err(e) => info!("[handle_l_book_offline] Error: {e}"),
                }

                match fetch_vlc_is_playing(port.clone(), address_player.clone()).await {
                    Ok(true) => {
                        current_time = book_wide_time;
                    }
                    Ok(false) => {
                        if current_track_idx + 1 < downloaded.tracks.len() {
                            info!("[handle_l_book_offline][Finished] Track {current_track_idx} finished, advancing to track {}", current_track_idx + 1);
                            let _ = update_current_time(book_wide_time, id_session.as_str());

                            let _ = quit_vlc(&address_player, &port);
                            pkill_vlc();
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                            current_track_idx += 1;
                            track_base_offset = downloaded.tracks[current_track_idx].start_offset.round() as u32;
                            let local_file_path = downloaded.tracks.get(current_track_idx).map(|t| t.local_path.clone());

                            spawn_track_vlc(
                                "handle_l_book_offline",
                                0,
                                port.clone(),
                                address_player.clone(),
                                String::new(),
                                None,
                                title.clone(),
                                title.clone(),
                                author.clone(),
                                String::new(),
                                program.clone(),
                                is_cvlc.clone(),
                                username.clone(),
                                id.clone(),
                                local_file_path,
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                            current_time = book_wide_time;
                            continue;
                        }

                        info!("[handle_l_book_offline][Finished] Track finished");
                        let _ = update_is_finished("1", id_session.as_str());
                        // Best-effort only - ignored whether the server is back or not.
                        let _ = update_media_progress2_book(&id, Some(&token), Some(book_wide_time), &downloaded.duration, true, server_address.clone()).await;
                        let _ = update_is_loop_break("1", username.as_str());
                        let _ = update_is_vlc_running("0", username.as_str());
                        break;
                    }
                    Err(_) => {
                        let _ = update_is_vlc_running("0", username.as_str());
                        info!("[handle_l_book_offline][Quit] Item {id} closed at {book_wide_time}s");
                        let _ = update_media_progress_book(&id, Some(&token), Some(book_wide_time), &downloaded.duration, server_address.clone()).await;
                        let _ = update_is_loop_break("1", username.as_str());
                        break;
                    }
                }
            }
            Ok(None) => {
                let _ = update_is_vlc_running("0", username.as_str());
                if got_real_data {
                    info!("[handle_l_book_offline][None] Item {id} closed at {current_time}s");
                    let _ = update_media_progress_book(&id, Some(&token), Some(current_time), &downloaded.duration, server_address.clone()).await;
                } else {
                    info!("[handle_l_book_offline][None] Item {id}: no real playback data was ever fetched, skipping progress sync");
                }
                let _ = update_is_loop_break("1", username.as_str());
                break;
            }
            Err(e) => {
                error!("[handle_l_book_offline][Err(e)]{e}");
                let _ = update_is_vlc_running("0", username.as_str());
                if got_real_data {
                    let _ = update_media_progress_book(&id, Some(&token), Some(current_time), &downloaded.duration, server_address.clone()).await;
                }
                let _ = update_is_loop_break("1", username.as_str());
                break;
            }
        }
    }
}
