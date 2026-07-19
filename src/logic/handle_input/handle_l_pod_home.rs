use crate::player::vlc::start_vlc::start_vlc;
use crate::player::vlc::fetch_vlc_data::{fetch_vlc_data, fetch_vlc_is_playing};
use crate::player::vlc::exec_nc::exec_nc;
use crate::api::me::update_media_progress::{update_media_progress_pod, update_media_progress2_pod};
use crate::api::library_items::play_lib_item_or_pod::post_start_playback_session_pod;
use crate::api::sessions::sync_open_session::sync_session;
use crate::api::sessions::close_open_session::close_session_without_send_prg_data;
use crate::utils::pop_up_message::clear_message;
use std::io::stdout;
use log::{info, error};
use crate::db::crud::{insert_listening_session, update_is_vlc_running, update_current_time, get_speed_rate, update_chapter, update_elapsed_time, update_is_finished, update_is_loop_break, get_is_podcast_autoplay};
use crate::utils::vlc_tcp_stream::vlc_tcp_stream;
use crate::player::vlc::quit_vlc::pkill_vlc;
use crate::utils::convert_seconds::progress_time_diff;


// handle l when is_podact is true for continue listening `AppView::Home`

pub async fn handle_l_pod_home(
    token: Option<&String>,
    ids_library_items: &[String],
    selected: Option<usize>,
    port: String,
    address_player: String,
    id_pod: Vec<String>,
    server_address: String,
    program: String,
    is_cvlc_term: String,
    username: String,

) {

    // need to pkill VLC for macos users
    pkill_vlc();

    let Some(token) = token else { return; };
    let Some(mut current_index) = selected else { return; };

    // Outer loop lets a finished episode advance to the next one in this same list
    // (Podcast Autoplay, toggled in Settings) without leaving this spawned task -
    // the main render loop already moved on as soon as this task was spawned, so
    // there's no synchronous caller left to hand control back to for a next episode.
    'episodes: loop {
        // id is id of the podcast  and id_pod_ep is the id id of the episode podcast
        let Some(id) = ids_library_items.get(current_index) else { break 'episodes; };
        let Some(id_pod_ep) = id_pod.get(current_index) else { break 'episodes; };

        if let Ok(info_item) = post_start_playback_session_pod(Some(token), id, id_pod_ep, server_address.clone()).await {

            // converting current time
            let mut current_time: u32 = info_item[0].parse::<f64>().unwrap().round() as u32;

            info!("[handle_l_pod_home][post_start_playback_session_pod] OK");
            info!("[handle_l_pod_home][post_start_playback_session_pod] Item {id_pod_ep} started at {current_time}s");
            // insert variables in databse (`listening_session` table) for sync session when app is quit
            let _ = insert_listening_session(
                info_item[3].clone(), // id_session
                id.clone(), // (id of the podcast, not the episode)
                current_time,  // current time
                info_item[2].clone(),
                id_pod_ep.clone(), // id of the podcast episode
                0, // elapsed time start at 0 seconds
                format!("{} | {}", info_item[5], info_item[4]), // "Episode Title | Podcast Title" - info_item[5] (displayTitle) is the actual episode title, info_item[4] (mediaMetadata.title) is the podcast's own title
                String::new(), // author not shown for podcasts
                true, // is_playback
                String::new(), // chapter
                String::new(), // chapters (not wired for podcasts yet)
            );


            // clone otherwise, these variable will  be consumed and not available anymore
            // for use outside start_vlc spawn
            let token_clone = token.clone();
            let port_clone = port.clone();
            let info_item_clone = info_item.clone() ;
            let server_address_clone = server_address.clone() ;
            let address_player_clone = address_player.clone() ;
            let username_clone = username.clone();
            let program_clone = program.clone();

            // Start VLC is launched in a spawn to allow fetch_vlc_data to start at the same time
            tokio::spawn(async move {
                // this info! is not the most reliable to know is VLC is really launched
                info!("[handle_l_pod_home][start_vlc] VLC successfully launched");
                start_vlc(
                    &info_item_clone[0], // current_time
                    &port_clone, // player port
                    address_player_clone, // player address
                    &info_item_clone[1], // content url
                    Some(&token_clone), //token
                    info_item_clone[4].clone(), //title
                    info_item_clone[5].clone(), // subtitle
                    info_item_clone[6].clone(), //title
                    server_address_clone.clone(), // server address
                    program_clone,
                    username_clone
                ).await;
            });

            if is_cvlc_term == "1" {
                let port_clone = port.clone();
                let address_player_clone = address_player.clone();
                tokio::spawn(async move {
                    exec_nc(&port_clone, address_player_clone).await;
                });
            }

            // clear loading message (from app.rs) when vlc is launched
            let mut stdout = stdout();
            let _ = clear_message(&mut stdout, 3);


            // Important, sleep time to 1s otherwise connection to vlc player will not have time to connect
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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
                        //  println!("last_curr: {}", last_current_time);
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
                                    let _ = update_media_progress_pod(id, Some(token), Some(data_fetched_from_vlc), &info_item[2], id_pod_ep, server_address.clone()).await;
                                    let _ = sync_session(Some(token), &info_item[3],Some(data_fetched_from_vlc), progress_sync, server_address.clone()).await;
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
                                info!("[handle_l_pod_home][Finished] Track finished");

                                // update is_finished in database (`listening_session` table)
                                let _ = update_is_finished("1", info_item[3].as_str());

                                let _ = close_session_without_send_prg_data(Some(token), &info_item[3],  server_address.clone()).await;
                                info!("[handle_l_pod_home][Finished] Session successfully closed");
                                let _ = update_media_progress2_pod(id, Some(token), Some(data_fetched_from_vlc), &info_item[2], is_finised, id_pod_ep, server_address.clone()).await;
                                info!("[handle_l_pod_home][Finished] VLC stopped");
                                info!("[handle_l_pod_home][Finished] Item {id_pod_ep} closed at {data_fetched_from_vlc}s");
                                let _ = update_is_loop_break("1", username.as_str());

                                let _ = update_is_vlc_running("0", username.as_str());

                                // Podcast Autoplay: if on, and there's a next episode in
                                // this same list, start it - otherwise stop here just like
                                // before this feature existed.
                                if get_is_podcast_autoplay(username.as_str()) == "1"
                                    && current_index + 1 < ids_library_items.len()
                                    && current_index + 1 < id_pod.len() {
                                        info!("[handle_l_pod_home][Finished] Autoplay is on, advancing to next episode");
                                        current_index += 1;
                                        continue 'episodes;
                                }
                                break 'episodes;
                            },
                            // `Err` means :  VLC is close (because if VLC is not playing
                            // anymore an error is send by `fetch_vlc_is_playing`).
                            // The track is not finished. VLC is just stopped by the user.
                            // Differ from the case above where the track reched the end.
                            Err(_e) => {
                                let _ = update_is_vlc_running("0", username.as_str());
                                info!("[handle_l_pod_home][Quit]");
                                // close session when VLC is quitted
                                let _ = close_session_without_send_prg_data(Some(token), &info_item[3],  server_address.clone()).await;
                                info!("[handle_l_pod_home][Quit] Session successfully closed");
                                // send one last time media progress (bug to retrieve media
                                // progress otherwise)
                                let _ = update_media_progress_pod(id, Some(token), Some(data_fetched_from_vlc), &info_item[2], id_pod_ep, server_address.clone()).await;
                                info!("[handle_l_pod_home][Quit] VLC closed");
                                info!("[handle_l_pod_home][Quit] Item {id_pod_ep} closed at {data_fetched_from_vlc}s");

                                //eprintln!("Error fetching play status: {}", e);
                                let _ = update_is_loop_break("1", username.as_str());
                                break 'episodes;
                            }
                        }

                    }
                    // when no data in fetched (generaly when VLC is launched and quit
                    // quickly) Indeed, in this case, data does not have enough time to be
                    // fetched
                    Ok(None) => {
                        let _ = update_is_vlc_running("0", username.as_str());
                        info!("[handle_l_pod_home][None]");
                        let _ = close_session_without_send_prg_data(Some(token), &info_item[3],  server_address.clone()).await;
                        info!("[handle_l_pod_home][None] Session successfully closed");
                        let _ = update_media_progress_pod(id, Some(token), Some(current_time), &info_item[2], id_pod_ep, server_address.clone()).await;
                        info!("[handle_l_pod_home][None] VLC closed");
                        info!("[handle_l_pod_home][None] Item {id} closed at {current_time}s");

                        let _ = update_is_loop_break("1", username.as_str());
                        break 'episodes; // Exit if no data available
                    }
                    Err(e) => {
                        error!("[handle_l_pod_home][Err(e)]{e}");
                        break 'episodes; // Exit on error
                    }
                }
            }
        } else {
            error!("[handle_l_pod_home] Failed to start playback session");
            eprintln!("Failed to start playback session");
            break 'episodes;
        }
    }
}
