use crate::player::vlc::start_vlc::start_vlc;
use crate::player::vlc::fetch_vlc_data::{fetch_vlc_data, fetch_vlc_is_playing};
use crate::player::vlc::exec_nc::exec_nc;
use crate::utils::pop_up_message::clear_message;
use crate::api::me::update_media_progress::{update_media_progress_book, update_media_progress2_book};
use crate::api::library_items::play_lib_item_or_pod::post_start_playback_session_book;
use crate::api::sessions::sync_open_session::sync_session;
use crate::api::sessions::close_open_session::close_session_without_send_prg_data;
use std::io::stdout;
use log::{info, error};
use crate::db::crud::{insert_listening_session, update_is_vlc_running, update_current_time, get_speed_rate, update_chapter, update_elapsed_time, update_is_finished, update_is_loop_break, get_download, get_listening_session};
use crate::db::database_struct::DownloadedItem;
use crate::utils::vlc_tcp_stream::vlc_tcp_stream;
use crate::player::vlc::quit_vlc::pkill_vlc;
use crate::utils::convert_seconds::progress_time_diff;

pub async fn handle_l_book(
    token: Option<&String>,
    ids_library_items: Vec<String>,
    selected: Option<usize>,
    port: String,
    address_player: String,
    server_address: String,
    program: String,
    is_cvlc_term: String,
    username: String,
) {

    // need to pkill VLC for macos users
    pkill_vlc();
  
    if let Some(index) = selected
        && let Some(id) = ids_library_items.get(index)
            && let Some(token) = token {
                // Checked up front so both branches below (online, and the offline
                // fallback if the session-start call fails) know whether a local copy
                // exists - see src/utils/download_cache.rs.
                let downloaded = get_download(&username, id);

                match post_start_playback_session_book(Some(token), id, server_address.clone()).await {
                    Err(e) => {
                        if let Some(downloaded) = downloaded {
                            info!("[handle_l_book] Couldn't start an online playback session ({e}) - falling back to the downloaded copy of {id}");
                            handle_l_book_offline(
                                id.clone(),
                                downloaded,
                                port,
                                address_player,
                                program,
                                username,
                                token.clone(),
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

                    // converting current time
                    let mut current_time: u32 = info_item[0].parse::<f64>().unwrap().round() as u32;

                    info!("[handle_l_book][post_start_playback_session_book] OK");
                    info!("[handle_l_book][post_start_playback_session_book] Item {id} started at {current_time}s");


                    // insert variables in databse (`listening_session` table) for sync session when app is quit
                    let _ = insert_listening_session(
                        info_item[3].clone(), // id_session
                        id.clone(), // id_item
                        current_time,  // current time
                        info_item[2].clone(), // total item duration
                        String::new(), // empty here, because it's for podcasts
                        0, // elapsed time start at 0 seconds
                        info_item[4].clone(), // title
                        info_item[6].clone(), // author
                        true, // is_playback
                        String::new(), // chapter
                        info_item[7].clone(), // chapters (JSON array of {id, title, start, end})
                        );
                        
                    // clone otherwise, these variable will  be consumed and not available anymore
                    // for use outside start_vlc spawn
                    let token_clone = token.clone();
                    let port_clone = port.clone();
                    let info_item_clone = info_item.clone() ;
                    let server_address_clone = server_address.clone() ;
                    let address_player_clone = address_player.clone() ;
                    let username_clone = username.clone();
                    let id_clone = id.clone();
                    // downloaded book, if any - play from the local copy instead of
                    // streaming, even though the server was reachable enough to start
                    // this session
                    let local_file_path = downloaded.map(|d| d.file_path);

                    // start_vlc is launched in a spawn to allow fetch_vlc_data to start at the same time
                    tokio::spawn(async move {
                        // this info! is not the most reliable to know is VLC is really launched
                        info!("[handle_l_book][start_vlc] VLC successfully launched");
                        if let Err(e) = start_vlc(
                            &info_item_clone[0], // current_time
                            &port_clone, // player port
                            address_player_clone, // player address
                            &info_item_clone[1], // content url
                            Some(&token_clone), //token
                            info_item_clone[4].clone(), //title
                            info_item_clone[5].clone(), // subtitle
                            info_item_clone[6].clone(), //title
                            server_address_clone.clone(), // server address
                            program.clone(),
                            username_clone,
                            id_clone,
                            local_file_path,
                            ).await {
                                error!("[handle_l_book][start_vlc] {e}");
                            }
                    });


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


                    // Important, sleep time to 1s minimum otherwise connection to vlc player will not have time to connect
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                    // init var for decide to send 0 sec in sync session if player is in pause
                    // 3 sec is not very "pro" but it's because i'm sure for this first iteration
                    //   data_fetched_from_vlc will not be = to 3 (because a little delay is given
                    //   before sync progress, in my case 5 secs, others apps a little bit more)
                    //   futhermore, in the worst case, if data_fetched_from_vlc is equal ti 3 for
                    //   the first iteration, it will shift the progress sync to 5 secondes
                    let mut last_current_time: u32 = 3;
                    let mut progress_sync: u32 = 3;

                    let _ = update_is_vlc_running("1", username.as_str());

                    let mut trigger = 1; 

                    loop {
                        match fetch_vlc_data(port.clone(), address_player.clone()).await {
                            Ok(Some(data_fetched_from_vlc)) => {
                                // println!("Fetched data: {}", data_fetched_from_vlc.to_string());

                                // update current_time in database (`listening_session` table)
                                let _ = update_current_time(data_fetched_from_vlc, info_item[3].as_str());

                                // Important, sleep time to 1s minimum, otherwise connection to vlc player will not have time to connect
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                // println!("last_curr: {}", last_current_time);
                                if data_fetched_from_vlc == last_current_time {
                                    progress_sync = 0; // the track is in pause
                                } else {
                                    let speed_rate_str = get_speed_rate(username.as_str());
                                    let speed_rate = speed_rate_str.parse::<f64>().unwrap_or(1.0);
                                    let current_time_adjusted = f64::from(current_time) / speed_rate; 
                                    let data_fetched_from_vlc_adjusted = f64::from(data_fetched_from_vlc) / speed_rate; 
                                    let diff = progress_time_diff(data_fetched_from_vlc_adjusted, current_time_adjusted);
                                    // if > 20 means that new current_time is not take into account
                                    // so we need to temporarly, put 1 sec if it happens (not the
                                    // most accurate...)
                                    // happen when a new jump/back of a chapter, or jump/back 10s
                                    // the difference is between data_fetched_from_vlc_adjusted,
                                    // and old currentitime_adjusted. This last one don't have time
                                    // to be the accurate version, because trigger is not equal to
                                    // 10 (so, it can't reach current_time = data_fetched_from_vlc in fetch_vlc_is_playing function bellow))
                                    if diff > 20 {
                                        progress_sync += 1;
                                    } else {
                                    progress_sync = diff;
                                    }
                                }
                                last_current_time = data_fetched_from_vlc;

                                // get current chapter
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
                                                let _ = sync_session(Some(token), &info_item[3],Some(data_fetched_from_vlc), progress_sync, server_address.clone()).await;
                                                let _ = update_media_progress_book(id, Some(token), Some(data_fetched_from_vlc), &info_item[2], server_address.clone()).await;
                                             
                                                // update elapsed_time in database (`listening_session` table)
                                                let _ = update_elapsed_time(progress_sync, info_item[3].as_str());

                                                current_time = data_fetched_from_vlc;
                                                progress_sync = 0;
                                                trigger = 0;

                                        } else if progress_sync != 0 {
                                            trigger += 1;
                                        } else if progress_sync == 0 {
                                            trigger += 0;
                                        }
                                    },
                                    // `Ok(false)` means that the track is stopped but VLC still
                                    // open. Allow to track when the audio reached the end. And
                                    // differ from the case where the user just close VLC
                                    // during a playing (in this case we don't want to mark the
                                    // track as finished)
                                    Ok(false) => {
                                        let is_finised = true;
                                        info!("[handle_l_book][Finished] Track finished");

                                        // update is_finished in database (`listening_session` table)
                                        let _ = update_is_finished("1", info_item[3].as_str());
                                        
                                        let _ = close_session_without_send_prg_data(Some(token), &info_item[3],  server_address.clone()).await;
                                        info!("[handle_l_book][Finished] Session successfully closed");
                                        let _ = update_media_progress2_book(id, Some(token), Some(data_fetched_from_vlc), &info_item[2], is_finised, server_address).await;
                                        info!("[handle_l_book][Finished] VLC stopped");
                                        info!("[handle_l_book][Finished] Item {id} closed at {data_fetched_from_vlc}s");
                                        let _ = update_is_loop_break("1", username.as_str());

                                        let _ = update_is_vlc_running("0", username.as_str());
                                        break; 
                                    },
                                    // `Err` means :  VLC is close (because if VLC is not playing
                                    // anymore an error is send by `fetch_vlc_is_playing`).
                                    // The track is not finished. VLC is just stopped by the user.
                                    // Differ from the case above where the track reched the end.
                                    Err(_) => {
                                        let _ = update_is_vlc_running("0", username.as_str());
                                        info!("[handle_l_book][Quit]");
                                        // close session when VLC is quitted
                                        let _ = close_session_without_send_prg_data(Some(token), &info_item[3],  server_address.clone()).await;
                                        info!("[handle_l_book][Quit] Session successfully closed");
                                        // send one last time media progress (bug to retrieve media
                                        // progress otherwise)
                                        let _ = update_media_progress_book(id, Some(token), Some(data_fetched_from_vlc), &info_item[2], server_address).await;
                                        info!("[handle_l_book][Quit] VLC closed");
                                        info!("[handle_l_book][Quit] Item {id} closed at {data_fetched_from_vlc}s");
                                        //eprintln!("Error fetching play status: {}", e);
                                        let _ = update_is_loop_break("1", username.as_str());
                                        break; 
                                    }
                                }

                            }
                            // when no data in fetched (generaly when VLC is launched and quit
                            // quickly) Indeed, in this case, data does not have enough time to be
                            // fetched
                            Ok(None) => {
                                let _ = update_is_vlc_running("0", username.as_str());
                                info!("[handle_l_book][None]");
                                let _ = close_session_without_send_prg_data(Some(token), &info_item[3],  server_address.clone()).await;
                                info!("[handle_l_book][None] Session successfully closed");
                                let _ = update_media_progress_book(id, Some(token), Some(current_time), &info_item[2], server_address.clone()).await;
                                info!("[handle_l_book][None] VLC closed");
                                info!("[handle_l_book][None] Item {id} closed at {current_time}s");

                                let _ = update_is_loop_break("1", username.as_str());
                                break; // Exit if no data available
                            }
                            Err(e) => {
                                error!("[handle_l_book][Err(e)]{e}");
                                let _ = update_is_vlc_running("0", username.as_str());
                                let _ = update_is_loop_break("1", username.as_str());
                                break; // Exit on error
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
async fn handle_l_book_offline(
    id: String,
    downloaded: DownloadedItem,
    port: String,
    address_player: String,
    program: String,
    username: String,
    token: String,
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
        String::new(),
    );

    let port_clone = port.clone();
    let address_player_clone = address_player.clone();
    let username_clone = username.clone();
    let id_clone = id.clone();
    let local_file_path = downloaded.file_path.clone();
    let title = downloaded.title.clone();
    let author = downloaded.author.clone();
    let current_time_str = current_time.to_string();

    tokio::spawn(async move {
        info!("[handle_l_book_offline][start_vlc] VLC launched against local file (offline)");
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
            author,
            String::new(),
            program,
            username_clone,
            id_clone,
            Some(local_file_path),
        ).await {
            error!("[handle_l_book_offline][start_vlc] {e}");
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
                    Err(e) => info!("[handle_l_book_offline] Error: {e}"),
                }

                match fetch_vlc_is_playing(port.clone(), address_player.clone()).await {
                    Ok(true) => {
                        current_time = data_fetched_from_vlc;
                    }
                    Ok(false) => {
                        info!("[handle_l_book_offline][Finished] Track finished");
                        let _ = update_is_finished("1", id_session.as_str());
                        // Best-effort only - ignored whether the server is back or not.
                        let _ = update_media_progress2_book(&id, Some(&token), Some(data_fetched_from_vlc), &downloaded.duration, true, server_address.clone()).await;
                        let _ = update_is_loop_break("1", username.as_str());
                        let _ = update_is_vlc_running("0", username.as_str());
                        break;
                    }
                    Err(_) => {
                        let _ = update_is_vlc_running("0", username.as_str());
                        info!("[handle_l_book_offline][Quit] Item {id} closed at {data_fetched_from_vlc}s");
                        let _ = update_media_progress_book(&id, Some(&token), Some(data_fetched_from_vlc), &downloaded.duration, server_address.clone()).await;
                        let _ = update_is_loop_break("1", username.as_str());
                        break;
                    }
                }
            }
            Ok(None) => {
                let _ = update_is_vlc_running("0", username.as_str());
                info!("[handle_l_book_offline][None] Item {id} closed at {current_time}s");
                let _ = update_media_progress_book(&id, Some(&token), Some(current_time), &downloaded.duration, server_address.clone()).await;
                let _ = update_is_loop_break("1", username.as_str());
                break;
            }
            Err(e) => {
                error!("[handle_l_book_offline][Err(e)]{e}");
                let _ = update_is_vlc_running("0", username.as_str());
                let _ = update_is_loop_break("1", username.as_str());
                break;
            }
        }
    }
}

