use crate::db::crud::{get_listening_session, update_is_vlc_launched_first_time, get_is_vlc_launched_first_time};
use crate::api::sessions::close_open_session::close_session_without_send_prg_data;
use log::info;
use crate::api::me::update_media_progress::{update_media_progress_book, update_media_progress2_book, update_media_progress_pod, update_media_progress2_pod};
use crate::player::vlc::quit_vlc::quit_vlc;
use crate::utils::exit_app::clean_exit;

// close and sync listening session before quit the app                
pub async fn sync_session_from_database(token: Option<String>, server_address: String, username: String, app_quit: bool, handle_key: &str, player_address: String, port: String) {

    // quit vlc before close and sync session (or close the app)
    let _ = quit_vlc(player_address.as_str(), port.as_str());

    match get_listening_session() {
        Ok(Some(session)) => {

            let _ = close_session_without_send_prg_data(
                token.as_ref(), 
                session.id_session.as_str(), 
                server_address.clone()).await;

            match handle_key {
                "Q" => info!("[handle_key (Q)][Quit] Session successfully closed"),
                "l" => info!("[handle_key (l)] Session successfully closed"),
                _ => {}
            }

            if session.id_pod.is_empty() {
                if session.is_finished {
                    let is_finished = true;
                    let _ = update_media_progress2_book(
                        session.id_item.as_str(), 
                        token.as_ref(), 
                        Some(session.current_time), 
                        &session.duration, 
                        is_finished, 
                        server_address).await;

                    match handle_key {
                        "Q" => info!("[handle_key (Q)][book][Quit] Item {} closed at {:?}s (finished)", session.id_item, session.current_time),
                        "l" => info!("[handle_key (l)] Item {} closed at {:?}s (finished)", session.id_item, session.current_time),
                        _ => {}
                    }
                } else {
                    let _ = update_media_progress_book(
                        session.id_item.as_str(), 
                        token.as_ref(), 
                        Some(session.current_time), 
                        &session.duration, 
                        server_address.clone()).await;

                    match handle_key {
                        "Q" => info!("[handle_key (Q)][book][Quit] Item {} closed at {:?}s (not finished)", session.id_item, session.current_time),
                        "l" => info!("[handle_key (l)] Item {} closed at {:?}s (not finished)", session.id_item, session.current_time),
                        _ => {}
                    }
                }

            } else {
                if session.is_finished {
                    let is_finished = true;
                    let _ = update_media_progress2_pod(
                        session.id_item.as_str(), 
                        token.as_ref(), 
                        Some(session.current_time), 
                        &session.duration, 
                        is_finished,
                        session.id_pod.as_str(), 
                        server_address.clone()).await;

                    match handle_key {
                        "Q" => info!("[handle_key (Q)][podcast][Quit] Item {} closed at {:?}s (finished)", session.id_pod, session.current_time),
                        "l" => info!("[handle_key (l)] Item {} closed at {:?}s (finished)", session.id_pod, session.current_time),
                        _ => {}
                    }
                } else {
                    let _ = update_media_progress_pod(
                        session.id_item.as_str(), 
                        token.as_ref(), 
                        Some(session.current_time), 
                        &session.duration, 
                        session.id_pod.as_str(), 
                        server_address.clone()).await;


                    match handle_key {
                        "Q" => info!("[handle_key (Q)][podcast][Quit] Item {} closed at {:?}s", session.id_pod, session.current_time),
                        "l" => info!("[handle_key (l)] Item {} closed at {:?}s", session.id_pod, session.current_time),
                        _ => {}
                    }
                }
            }

            if app_quit {
                // update is_vlc_launched_first_time
                let _ = update_is_vlc_launched_first_time("1", username.as_str());
                let value = get_is_vlc_launched_first_time(username.as_str());
                info!("[exit][is_vlc_launched_first_time] {value}");

                // exit app
                info!("App successfully quit");
                clean_exit();

            }
        }

        Ok(None) => {
            let value = get_is_vlc_launched_first_time(username.as_str());
            if value == "1" {
            info!("[handle_key] Quit with no listening session");
            clean_exit();
            } else {
                info!("[handle_key] First session launched");
            }
        }        
        Err(e) => {
            info!("[handle_key] Error during fetching session: {e:?}");
        }    
    }
}

