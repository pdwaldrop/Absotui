use crate::api::utils::collect_personalized_view::{collect_titles_cnt_list, collect_auth_names_cnt_list, collect_pub_year_cnt_list, collect_duration_cnt_list, collect_desc_cnt_list, collect_ids_cnt_list};
use crate::api::utils::collect_personalized_view_pod::{collect_ids_pod_cnt_list, collect_titles_cnt_list_pod, collect_ids_ep_pod_cnt_list, collect_subtitles_pod_cnt_list, collect_nums_ep_pod_cnt_list, collect_seasons_pod_cnt_list, collect_authors_pod_cnt_list, collect_descs_pod_cnt_list, collect_titles_pod_cnt_list, collect_durations_pod_cnt_list, collect_progress_pod_cnt_list, collect_published_at_pod_cnt_list, collect_embedded_cover_ino_pod_cnt_list};
use crate::api::utils::collect_get_all_books::{collect_titles_library, collect_ids_library, collect_auth_names_library, collect_auth_names_library_pod, collect_published_year_library, collect_desc_library, collect_duration_library};
use crate::api::utils::collect_get_pod_ep::{collect_titles_pod_ep, collect_ids_pod_ep, collect_subtitles_pod_ep, collect_seasons_pod_ep, collect_episodes_pod_ep, collect_authors_pod_ep, collect_descs_pod_ep, collect_titles_pod, collect_durations_pod_ep};
use crate::api::utils::collect_get_all_libraries::{collect_library_names, collect_media_types, collect_library_ids};
use crate::api::utils::collect_get_media_progress::{collect_progress_percentage_book, collect_is_finished_book, collect_current_time_prg};
use crate::api::me::get_media_progress::get_book_progress;
use crate::api::me::update_media_progress::update_media_progress2_pod;
use crate::api::libraries::get_library_perso_view::get_continue_listening;
use crate::api::libraries::get_library_perso_view_pod::{get_new_and_unfinished_pod, Chapter};
use crate::api::libraries::get_all_books::get_all_books;
use crate::api::libraries::get_all_libraries::get_all_libraries;
use crate::api::library_items::get_pod_ep::get_pod_ep;
use crate::logic::handle_input::handle_l_book::handle_l_book;
use crate::logic::handle_input::handle_l_pod::handle_l_pod;
use crate::logic::handle_input::handle_l_pod_home::handle_l_pod_home;
use crate::config::{ConfigFile, load_config};
use crate::db::crud::{get_is_show_key_bindings, update_is_show_key_bindings, get_is_speed_adjusted_time, update_is_speed_adjusted_time, update_is_podcast_autoplay, update_is_vlc_running, delete_user, update_id_selected_lib, get_listening_session, get_is_vlc_running, update_is_per_item_speed, update_is_finished};
use crate::db::database_struct::Database;
use crate::utils::convert_seconds::convert_seconds;
use color_eyre::Result;
use log::warn;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    widgets::ListState,
};
use crate::utils::pop_up_message::pop_message;
use crate::utils::changelog::changelog;
use crate::utils::encrypt_token::decrypt_token;
use std::io::stdout;
use crate::player::vlc::quit_vlc::{quit_vlc, pkill_vlc};
use crate::logic::sync_session::sync_session_from_database::sync_session_from_database;
use crate::logic::sync_session::wait_prev_session_finished::wait_prev_session_finished;
use crate::player::integrated::handle_key_player::{handle_key_player, seek_to_absolute_time};
use crate::utils::check_update::check_update;

// A single row of the (book-mode) Home list, once the currently-playing book's chapter
// list has been spliced in as extra rows. Kept in sync between rendering (tui.rs) and
// input handling (this file) by always going through `App::build_home_rows`.
#[derive(Clone)]
pub enum HomeRow {
    Book(usize),
    Chapter { book_index: usize, chapter: Chapter },
}

pub enum AppView {
    Home,
    Library,
    SearchBook,
    PodcastEpisode,
    Settings,
    SettingsAccount,
    SettingsLibrary,
    SettingsAbout,
    SettingsUpdateUninstall,
    SettingsAutoplay,
    SettingsPerItemSpeed
}

pub struct App {
    pub view_state: AppView,
    // Set when the user picks a different library in Settings > Library. `App` can't
    // reinitialize itself (that's an async operation, and this struct's own methods
    // are sync), so this just signals the main loop to do the same full reload/reinit
    // it already does for the `R` key, landing back on Home in the newly selected library.
    pub library_needs_reload: bool,
    // Whether the currently-playing book's chapter list is expanded inline under its row
    // in Continue Listening. Session-local only (not persisted) - matches the pattern used
    // by other ephemeral view toggles like `podcast_sort_newest_first`.
    pub is_chapter_list_expanded: bool,
    pub database: Database,
    pub id_selected_lib: String,
    pub token: Option<String>,
    pub should_exit: bool,
    pub list_state_cnt_list: ListState,
    pub list_state_library: ListState,
    pub list_state_search_results: ListState,
    pub list_state_pod_ep: ListState,
    pub list_state_settings: ListState,
    pub list_state_settings_account: ListState,
    pub list_state_settings_library: ListState,
    pub list_state_settings_about: ListState,
    pub list_state_settings_update_uninstall: ListState,
    pub list_state_settings_autoplay: ListState,
    pub list_state_settings_per_item_speed: ListState,
    pub _titles_cnt_list: Vec<String>,
    pub auth_names_cnt_list: Vec<String>,
    pub pub_year_cnt_list: Vec<String>,
    pub duration_cnt_list: Vec<f64>,
    pub desc_cnt_list: Vec<String>,
    pub _ids_cnt_list: Vec<String>,
    pub titles_library: Vec<String>,
    pub ids_library: Vec<String>,
    pub auth_names_library: Vec<String>,
    pub ids_search_book: Vec<String>,
    pub search_query: String,
    pub search_mode: bool,
    pub is_podcast: bool,
    pub all_titles_pod_ep: Vec<Vec<String>>,
    pub all_ids_pod_ep: Vec<Vec<String>>,
    pub all_subtitles_pod_ep: Vec<Vec<String>>,
    pub all_seasons_pod_ep: Vec<Vec<String>>,
    pub all_episodes_pod_ep: Vec<Vec<String>>,
    pub all_authors_pod_ep: Vec<Vec<String>>,
    pub all_descs_pod_ep: Vec<Vec<String>>,
    pub all_titles_pod: Vec<Vec<String>>,
    pub all_durations_pod_ep: Vec<Vec<String>>,
    pub titles_pod_ep: Vec<String>,
    pub ids_pod_ep: Vec<String>,
    pub ids_pod_ep_search: Vec<String>,
    pub subtitles_pod_ep: Vec<String>,
    pub seasons_pod_ep: Vec<String>,
    pub episodes_pod_ep: Vec<String>,
    pub authors_pod_ep: Vec<String>,
    pub descs_pod_ep: Vec<String>,
    pub titles_pod: Vec<String>,
    pub durations_pod_ep: Vec<String>,
    pub ids_ep_cnt_list: Vec<String>,
    pub all_titles_pod_ep_search: Vec<Vec<String>>,
    pub titles_pod_ep_search: Vec<String>,
    pub is_from_search_pod: bool,
    pub ids_library_pod_search: Vec<String>,
    pub all_ids_pod_ep_search: Vec<Vec<String>>,
    pub libraries_names: Vec<String>,
    pub media_types: Vec<String>,
    pub libraries_ids: Vec<String>,
    pub library_name: String,
    pub media_type: String,
    pub lib_name_type: String,
    pub settings: Vec<String>,
    pub all_usernames: Vec<String>,
    pub all_server_addresses: Vec<String>,
    pub username: String,
    pub server_address: String,
    pub server_address_pretty: String,
    pub scroll_offset: u16,
    pub subtitles_pod_cnt_list: Vec<String>,
    pub nums_ep_pod_cnt_list: Vec<String>,
    pub seasons_pod_cnt_list: Vec<String>,
    pub authors_pod_cnt_list: Vec<String>,
    pub descs_pod_cnt_list: Vec<String>,
    pub titles_pod_cnt_list: Vec<String>,
    pub durations_pod_cnt_list: Vec<String>,
    pub podcast_progress_cnt_list: Vec<(f64, f64, f32)>,
    pub podcast_published_at_cnt_list: Vec<i64>,
    // `ino` of the episode's audio file when it's worth checking for embedded cover art
    // (MP3 + ffprobe detected a picture stream in it) - None otherwise, same index as
    // `ids_ep_cnt_list`. See collect_embedded_cover_ino_pod_cnt_list.
    pub episode_embedded_cover_ino_cnt_list: Vec<Option<String>>,
    pub podcast_sort_newest_first: bool,
    // Marquee-scroll state for a truncated title on the currently selected list row.
    // Ticks forward on a timer (not every render) so scroll speed stays constant
    // regardless of render rate; resets whenever the selection moves to a new row.
    pub title_scroll_offset: u32,
    pub title_scroll_last_tick: std::time::Instant,
    pub title_scroll_selected: Option<usize>,
    pub published_year_library: Vec<String>,
    pub desc_library: Vec<String>,
    pub duration_library: Vec<f64>,
    pub auth_names_library_pod: Vec<String>,
    pub subtitles_pod_ep_search: Vec<String>,
    pub seasons_pod_ep_search: Vec<String>,
    pub episodes_pod_ep_search: Vec<String>,
    pub authors_pod_ep_search: Vec<String>,
    pub descs_pod_ep_search: Vec<String>,
    pub titles_pod_search: Vec<String>,
    pub durations_pod_ep_search: Vec<String>,
    pub all_subtitles_pod_ep_search: Vec<Vec<String>>,
    pub all_seasons_pod_ep_search: Vec<Vec<String>>,
    pub all_episodes_pod_ep_search: Vec<Vec<String>>,
    pub all_authors_pod_ep_search: Vec<Vec<String>>,
    pub all_descs_pod_ep_search: Vec<Vec<String>>,
    pub all_titles_pod_search: Vec<Vec<String>>,
    pub all_durations_pod_ep_search: Vec<Vec<String>>,
    pub auth_names_pod_search_book: Vec<String>,
    pub auth_names_search_book: Vec<String>,
    pub published_year_library_search_book: Vec<String>,
    pub desc_library_search_book: Vec<String>,
    pub duration_library_search_book: Vec<f64>,
    pub book_progress_cnt_list: Vec<Vec<String>>,
    pub book_progress_cnt_list_cur_time: Vec<Vec<f64>>,
//    pub book_progress_library: Vec<Vec<String>>,
//    pub book_progress_library_cur_time: Vec<Vec<f64>>,
    pub book_progress_search_book: Vec<Vec<String>>,
    pub book_progress_search_book_cur_time: Vec<Vec<f64>>,
    pub is_cvlc: String,
    pub is_cvlc_term: String,
    pub start_vlc_program: String,
    pub config: ConfigFile,
    pub changelog: String,
    pub update_msg: String,
    pub podcast_home_last_refresh: std::time::Instant,
    // None if the terminal doesn't support any image protocol (Kitty/Sixel/iTerm2) -
    // queried once at startup via Picker::from_query_stdio().
    pub image_picker: Option<ratatui_image::picker::Picker>,
    // The decoded cover currently being shown, and which item id it belongs to - compared
    // against the selected row each render to know when to load a different cover.
    pub cover_protocol: Option<ratatui_image::protocol::StatefulProtocol>,
    pub cover_loaded_for_id: Option<String>,
    // Item ids a background fetch has already been kicked off for, so repeatedly
    // rendering the same selection while the fetch is in flight doesn't spawn duplicates.
    pub cover_fetch_requested: std::collections::HashSet<String>,
}

// Bundles what render_home needs for the podcast "New & Unfinished" list, so the same
// fetch logic can run both at initial load (App::new) and on a periodic refresh
// without duplicating it or needing to reconstruct the whole App.
struct PodcastHomeData {
    ids: Vec<String>,
    titles: Vec<String>,
    ids_ep: Vec<String>,
    subtitles: Vec<String>,
    nums_ep: Vec<String>,
    seasons: Vec<String>,
    authors: Vec<String>,
    descs: Vec<String>,
    titles_pod: Vec<String>,
    durations: Vec<String>,
    // (current_time, duration_seconds, percent) per episode - raw values, formatted for
    // display in render_home like the book progress text.
    progress: Vec<(f64, f64, f32)>,
    published_at: Vec<i64>,
    // `ino` of the episode's audio file, only when it's worth checking for embedded
    // cover art - see collect_embedded_cover_ino_pod_cnt_list.
    embedded_cover_ino: Vec<Option<String>>,
}

async fn fetch_podcast_home_data(token: &str, server_address: String, id_selected_lib: &String, newest_first: bool, username: &str) -> Result<PodcastHomeData> {
    let continue_listening_pod = get_new_and_unfinished_pod(token, server_address.clone(), id_selected_lib).await?;
    let mut data = PodcastHomeData {
        ids: collect_ids_pod_cnt_list(&continue_listening_pod).await,
        titles: collect_titles_cnt_list_pod(&continue_listening_pod).await,
        ids_ep: collect_ids_ep_pod_cnt_list(&continue_listening_pod).await,
        subtitles: collect_subtitles_pod_cnt_list(&continue_listening_pod).await,
        nums_ep: collect_nums_ep_pod_cnt_list(&continue_listening_pod).await,
        seasons: collect_seasons_pod_cnt_list(&continue_listening_pod).await,
        authors: collect_authors_pod_cnt_list(&continue_listening_pod).await,
        descs: collect_descs_pod_cnt_list(&continue_listening_pod).await,
        titles_pod: collect_titles_pod_cnt_list(&continue_listening_pod).await,
        durations: collect_durations_pod_cnt_list(&continue_listening_pod).await,
        progress: collect_progress_pod_cnt_list(&continue_listening_pod).await,
        published_at: collect_published_at_pod_cnt_list(&continue_listening_pod).await,
        embedded_cover_ino: collect_embedded_cover_ino_pod_cnt_list(&continue_listening_pod).await,
    };

    let mut order: Vec<usize> = (0..data.published_at.len()).collect();
    if newest_first {
        order.sort_by_key(|&i| std::cmp::Reverse(data.published_at[i]));
    } else {
        order.sort_by_key(|&i| data.published_at[i]);
    }
    data.ids = order.iter().map(|&i| data.ids[i].clone()).collect();
    data.titles = order.iter().map(|&i| data.titles[i].clone()).collect();
    data.ids_ep = order.iter().map(|&i| data.ids_ep[i].clone()).collect();
    data.subtitles = order.iter().map(|&i| data.subtitles[i].clone()).collect();
    data.nums_ep = order.iter().map(|&i| data.nums_ep[i].clone()).collect();
    data.seasons = order.iter().map(|&i| data.seasons[i].clone()).collect();
    data.authors = order.iter().map(|&i| data.authors[i].clone()).collect();
    data.descs = order.iter().map(|&i| data.descs[i].clone()).collect();
    data.titles_pod = order.iter().map(|&i| data.titles_pod[i].clone()).collect();
    data.durations = order.iter().map(|&i| data.durations[i].clone()).collect();
    data.progress = order.iter().map(|&i| data.progress[i]).collect();
    data.published_at = order.iter().map(|&i| data.published_at[i]).collect();
    data.embedded_cover_ino = order.iter().map(|&i| data.embedded_cover_ino[i].clone()).collect();

    pin_now_playing_episode(&mut data, username);

    Ok(data)
}

// If the actively-playing podcast episode isn't in the freshly-fetched "New &
// Unfinished" list, pin it at the top instead of letting it silently vanish from view.
// This happens for real: a freshly-autoplayed episode only joins the server's
// "Continue Listening" shelf once this app's own periodic progress sync reaches it
// (every ~10s of active polling), so there's a window where the episode that's
// actually playing isn't in either server shelf `get_new_and_unfinished_pod` reads from
// - during which a user who can't see it playing may reselect it, thinking it's
// unplayed, which used to restart the same session.
//
// Deliberately NOT a network fetch for full metadata: this runs on the main render
// loop's own periodic refresh (`refresh_podcast_home_if_stale`), not a background task,
// so any network call here blocks rendering and key handling for however long it takes
// - and it would fire almost every time right after an autoplay transition, exactly
// when this same propagation delay is in effect. Instead this reuses what's already
// sitting in the local session row: `title` is stored as "Episode Title | Podcast
// Title" (see the `insert_listening_session` call sites), split back apart here rather
// than re-fetched. Subtitle/author/description/season/episode-number are left blank for
// this pinned row - they only matter if the user selects it, and self-correct as soon
// as the server's shelves catch up and a normal fetch picks the episode up with full
// data.
fn pin_now_playing_episode(data: &mut PodcastHomeData, username: &str) {
    // The listening_session row lingers indefinitely after playback ends - nothing
    // clears it, it's only ever replaced by the next `insert_listening_session` call -
    // so a non-empty id_pod alone doesn't mean anything is actually still playing. Gate
    // on is_vlc_running too, or every fresh launch/refresh would pin whichever podcast
    // episode was last listened to, ever, even if that was days ago - a phantom "now
    // playing" row that pushes everything else down a line for nothing.
    if get_is_vlc_running(username) != "1" {
        return;
    }

    let Some(session) = get_listening_session().ok().flatten() else { return };
    if session.id_pod.is_empty() || data.ids_ep.contains(&session.id_pod) {
        return;
    }

    let (episode_title, podcast_title) = session.title.split_once(" | ")
        .map(|(ep, pod)| (ep.to_string(), pod.to_string()))
        .unwrap_or_else(|| (session.title.clone(), String::new()));
    let duration = session.duration.parse::<f64>().unwrap_or(0.0);

    data.ids.insert(0, session.id_item.clone());
    data.titles.insert(0, episode_title);
    data.ids_ep.insert(0, session.id_pod.clone());
    data.subtitles.insert(0, String::new());
    data.nums_ep.insert(0, String::new());
    data.seasons.insert(0, String::new());
    data.authors.insert(0, String::new());
    data.descs.insert(0, String::new());
    data.titles_pod.insert(0, podcast_title);
    data.durations.insert(0, convert_seconds(vec![duration]).into_iter().next().unwrap_or_default());
    data.progress.insert(0, (session.current_time as f64, duration, 0.0));
    // Pinned regardless of natural sort position, so its published_at doesn't matter -
    // i64::MAX just documents that it was never meant to be re-sorted.
    data.published_at.insert(0, i64::MAX);
    data.embedded_cover_ino.insert(0, None);
}

/// Init app
impl App {
    pub async fn new() -> Result<Self> {

        // init config
        let config = load_config()?;

        // init database from Database struct
        let database = Database::new().await?;

        // init changelog
        let changelog = changelog();


        // retrieve crypted token from database
        let mut token: String = String::new();
        if let Some(var_token) = database.default_usr.get(2) {
            token = var_token.clone();
        }
        match decrypt_token(token.as_str()) {
            Ok(decrypted_token) => {
                token = decrypted_token;
                //info!("Token successfully decrypted")
            }
            Err(e) => {
                println!("Error: {e}");
            }
        }


        // init server_address
        let mut _server_address: String = String::new();
        if let Some(var_server_address) = database.default_usr.get(1) {
            _server_address = var_server_address.clone();
        }

        // init id_selected_lib
        let mut id_selected_lib: String = String::new();
        if let Some(var_id_selected_lib) = database.default_usr.get(5) {
            id_selected_lib = var_id_selected_lib.clone();
        }

        // init current username
        let mut username: String = String::new();
        if let Some(var_username) = database.default_usr.first() {
            username = var_username.clone();
        }

        // init server address (without prefix)
        let mut server_address: String = String::new();
        let mut server_address_pretty: String = String::new();
        if let Some(var_server_address) = database.default_usr.get(1) {
            server_address = var_server_address.clone();

            // Remove "http://" or "https://"
            if let Some(stripped) = server_address.strip_prefix("http://") {
                server_address_pretty = stripped.to_string();
            } else if let Some(stripped) = server_address.strip_prefix("https://") {
                server_address_pretty = stripped.to_string();
            }
        }

        // init for `Libraries` (get all Libraries (shelf), can be a podcast or book type)
        let all_libraries = get_all_libraries(&token, server_address.clone()).await?;
        let libraries_names = collect_library_names(&all_libraries).await; // all the libraries names of the user ex : {name1, name2}
    let media_types = collect_media_types(&all_libraries).await; // all media type of libraries ex : {book, podcast}
    let libraries_ids = collect_library_ids(&all_libraries).await; // all all libraries ids
    let mut library_name = String::new(); // library name of the selected library
    let mut media_type = String::new(); // media type of the selected library

    let target = id_selected_lib.clone();

    // retrieve name and mediatype of the current librarie
    if let Some(index) = libraries_ids.iter().position(|x| x == &target) {
        library_name = libraries_names[index].clone();
        media_type = media_types[index].clone();
    }         
    let lib_name_type = format!("📖 {library_name} ({media_type})");

    // init is_podcast
    let is_podcast = media_type == "podcast";

    // init for `Home` (continue listening)
    let mut _titles_cnt_list: Vec<String> = Vec::new();
    let mut auth_names_cnt_list: Vec<String> = Vec::new();
    let mut pub_year_cnt_list: Vec<String> = Vec::new();
    let mut duration_cnt_list: Vec<f64> = Vec::new();
    let mut desc_cnt_list: Vec<String> = Vec::new();
    let mut _ids_cnt_list: Vec<String> = Vec::new();
    let mut ids_ep_cnt_list: Vec<String> = Vec::new();
    let mut subtitles_pod_cnt_list: Vec<String> = Vec::new();
    let mut nums_ep_pod_cnt_list: Vec<String> = Vec::new();
    let mut seasons_pod_cnt_list: Vec<String> = Vec::new();
    let mut authors_pod_cnt_list: Vec<String> = Vec::new();
    let mut descs_pod_cnt_list: Vec<String> = Vec::new();
    let mut titles_pod_cnt_list: Vec<String> = Vec::new();
    let mut durations_pod_cnt_list: Vec<String> = Vec::new();
    let mut podcast_progress_cnt_list: Vec<(f64, f64, f32)> = Vec::new();
    let mut podcast_published_at_cnt_list: Vec<i64> = Vec::new();
    let mut episode_embedded_cover_ino_cnt_list: Vec<Option<String>> = Vec::new();
    let podcast_sort_newest_first = true;
    let mut book_progress_cnt_list: Vec<Vec<String>> = Vec::new();
    let mut book_progress_cnt_list_cur_time: Vec<Vec<f64>> = Vec::new();

    if is_podcast {
        // init for `Home` (new & unfinished episodes) for podcasts
        let data = fetch_podcast_home_data(&token, server_address.clone(), &id_selected_lib, podcast_sort_newest_first, &username).await?;
        _ids_cnt_list = data.ids;
        _titles_cnt_list = data.titles;
        ids_ep_cnt_list = data.ids_ep;
        subtitles_pod_cnt_list = data.subtitles;
        nums_ep_pod_cnt_list = data.nums_ep;
        seasons_pod_cnt_list = data.seasons;
        authors_pod_cnt_list = data.authors;
        descs_pod_cnt_list = data.descs;
        titles_pod_cnt_list = data.titles_pod;
        durations_pod_cnt_list = data.durations;
        podcast_progress_cnt_list = data.progress;
        podcast_published_at_cnt_list = data.published_at;
        episode_embedded_cover_ino_cnt_list = data.embedded_cover_ino;
    }
    else {
        // init for  `Home` (continue listening) for books
        let continue_listening = get_continue_listening(&token, server_address.clone(), &id_selected_lib.clone()).await?;
        _titles_cnt_list = collect_titles_cnt_list(&continue_listening).await;
        auth_names_cnt_list = collect_auth_names_cnt_list(&continue_listening).await;
        pub_year_cnt_list = collect_pub_year_cnt_list(&continue_listening).await;
        duration_cnt_list = collect_duration_cnt_list(&continue_listening).await;
        desc_cnt_list = collect_desc_cnt_list(&continue_listening).await;
        _ids_cnt_list = collect_ids_cnt_list(&continue_listening).await;
        for id in _ids_cnt_list.clone() {
            match get_book_progress(&token, &id, server_address.clone()).await {
                Ok(val) => {
                    let mut values: Vec<String> = Vec::new();
                    let mut values_f64: Vec<f64> = Vec::new();
                    values.push(collect_progress_percentage_book(&val).await);
                    values.push(collect_is_finished_book(&val).await);
                    values_f64.push(collect_current_time_prg(&val).await);
                    book_progress_cnt_list.push(values);
                    book_progress_cnt_list_cur_time.push(values_f64);
                }
                Err(e) => {
                    // This can genuinely mean "never started" (server 404s with no
                    // progress record), but it could also be a real request failure -
                    // logged so the two cases can be told apart.
                    warn!("[get_book_progress] item {id} - treating as not started: {e}");
                    let mut values: Vec<String> = Vec::new();
                    let mut values_f64: Vec<f64> = Vec::new();
                    values.push(" N/A".to_string());
                    values.push(" N/A".to_string());
                    values_f64.push(0.0);
                    book_progress_cnt_list.push(values);
                    book_progress_cnt_list_cur_time.push(values_f64);
                }
            }}}

    // None if the terminal doesn't support any image protocol - cover images just won't
    // be shown, falling back to text-only description panels everywhere.
    let image_picker = ratatui_image::picker::Picker::from_query_stdio().ok();

    //init for `Library ` (all books  or podcasts of a Library (shelf))
    let all_books = get_all_books(&token, &id_selected_lib, server_address.clone()).await?;
    let titles_library = collect_titles_library(&all_books).await;
    let ids_library = collect_ids_library(&all_books).await;
    let auth_names_library = collect_auth_names_library(&all_books).await; // for a book
    let auth_names_library_pod = collect_auth_names_library_pod(&all_books).await; // for a podcast
    let published_year_library = collect_published_year_library(&all_books).await;
    let desc_library = collect_desc_library(&all_books).await;
    let duration_library = collect_duration_library(&all_books).await;
//    let mut book_progress_library: Vec<Vec<String>> = Vec::new();
//    let mut book_progress_library_cur_time: Vec<Vec<f64>> = Vec::new();
//    if !is_podcast{
//        for id in _ids_cnt_list.clone() {
//            if let Ok(val) = get_book_progress(&token, &id, server_address.clone()).await {
//                let mut values: Vec<String> = Vec::new();
//                let mut values_f64: Vec<f64> = Vec::new();
//                values.push(format!(" {}%,",collect_progress_percentage_book(&val).await));
//                values.push(format!(" {}",collect_is_finished_book(&val).await));
//                values_f64.push(collect_current_time_prg(&val).await);
//                book_progress_library.push(values);
//                book_progress_library_cur_time.push(values_f64);
//                
//            } else {
//                // if the book is not starded, `get book progress` is not fetched
//                // so the empty values are handled here : 
//                // avoid an out of bound panick
//                let mut values: Vec<String> = Vec::new();
//                let mut values_f64: Vec<f64> = Vec::new();
//                values.push(format!(" Not started yet"));
//                values.push(format!(""));
//                values_f64.push(0.0);
//                book_progress_library.push(values);
//                book_progress_library_cur_time.push(values_f64);
//            }
//        }
//    }            

    // init for `SearchBook`

    let ids_search_book: Vec<String> = Vec::new();
    let _auth_names_pod_search_book: Vec<String> = Vec::new();
    let _auth_names_search_book: Vec<String> = Vec::new();
    let _published_year_library_search_book: Vec<String> = Vec::new();
    let _desc_library_search_book: Vec<String> = Vec::new();
    let auth_names_search_book: Vec<String> = Vec::new();
    let auth_names_pod_search_book: Vec<String> = Vec::new();
    let published_year_library_search_book: Vec<String> = Vec::new();
    let desc_library_search_book: Vec<String> = Vec::new();
    let duration_library_search_book: Vec<f64> = Vec::new();
    let book_progress_search_book: Vec<Vec<String>> = Vec::new(); 
    let book_progress_search_book_cur_time: Vec<Vec<f64>> = Vec::new(); 
    let search_mode = false;
    let search_query = "  ".to_string();
    let all_titles_pod_ep_search: Vec<Vec<String>> = Vec::new(); // init in tui.rs in render search book function
    let all_ids_pod_ep_search: Vec<Vec<String>> = Vec::new(); 
    let all_subtitles_pod_ep_search: Vec<Vec<String>> = Vec::new(); 
    let all_seasons_pod_ep_search: Vec<Vec<String>> = Vec::new(); 
    let all_episodes_pod_ep_search: Vec<Vec<String>> = Vec::new(); 
    let all_authors_pod_ep_search: Vec<Vec<String>> = Vec::new(); 
    let all_descs_pod_ep_search: Vec<Vec<String>> = Vec::new(); 
    let all_titles_pod_search: Vec<Vec<String>> = Vec::new(); 
    let all_durations_pod_ep_search: Vec<Vec<String>> = Vec::new(); 
    let titles_pod_ep_search: Vec<String> = Vec::new();
    let ids_library_pod_search: Vec<String> = Vec::new(); // library because we take index of library
    let subtitles_pod_ep_search: Vec<String> = Vec::new();
    let seasons_pod_ep_search: Vec<String> = Vec::new();
    let episodes_pod_ep_search: Vec<String> = Vec::new();
    let authors_pod_ep_search: Vec<String> = Vec::new();
    let descs_pod_ep_search: Vec<String> = Vec::new();
    let titles_pod_search: Vec<String> = Vec::new();
    let durations_pod_ep_search: Vec<String> = Vec::new();
    let is_from_search_pod = false;



    //init for `PodcastEpisode`
    let mut all_titles_pod_ep: Vec<Vec<String>> = Vec::new(); // fetch titles for all podcast episodes. Ex: {titles_pod1_ep1, title_pod1_ep2}, {titles_pod2_ep1, title_pod2_ep2} 
    let mut all_ids_pod_ep: Vec<Vec<String>> = Vec::new();
    let mut all_subtitles_pod_ep: Vec<Vec<String>> = Vec::new();
    let mut all_seasons_pod_ep: Vec<Vec<String>> = Vec::new();
    let mut all_episodes_pod_ep: Vec<Vec<String>> = Vec::new();
    let mut all_authors_pod_ep: Vec<Vec<String>> = Vec::new();
    let mut all_descs_pod_ep: Vec<Vec<String>> = Vec::new();
    let mut all_titles_pod: Vec<Vec<String>> = Vec::new(); // fetch title of a podcast (not episode)
    let mut all_durations_pod_ep: Vec<Vec<String>> = Vec::new();
    let titles_pod_ep: Vec<String> = Vec::new(); // fetch episode titles for a podcast. {titles_pod1_ep1, title_pod1_ep2} 
    let ids_pod_ep: Vec<String> = Vec::new();
    let ids_pod_ep_search: Vec<String> = Vec::new();
    let subtitles_pod_ep: Vec<String> = Vec::new();
    let seasons_pod_ep: Vec<String> = Vec::new();
    let episodes_pod_ep: Vec<String> = Vec::new();
    let authors_pod_ep: Vec<String> = Vec::new();
    let descs_pod_ep: Vec<String> = Vec::new();
    let titles_pod: Vec<String> = Vec::new();
    let durations_pod_ep: Vec<String> = Vec::new();

    if is_podcast {
    for id_library in &ids_library
    {let podcast_episode = get_pod_ep(&token, server_address.clone(), id_library.as_str()).await?;
        let title = collect_titles_pod_ep(&podcast_episode).await;
        all_titles_pod_ep.push(title);
        let id = collect_ids_pod_ep(&podcast_episode).await;
        all_ids_pod_ep.push(id);
        let sub = collect_subtitles_pod_ep(&podcast_episode).await;
        all_subtitles_pod_ep.push(sub);
        let seasons = collect_seasons_pod_ep(&podcast_episode).await;
        all_seasons_pod_ep.push(seasons);
        let numep = collect_episodes_pod_ep(&podcast_episode).await;
        all_episodes_pod_ep.push(numep);
        let authors = collect_authors_pod_ep(&podcast_episode).await;
        all_authors_pod_ep.push(authors);
        let desc = collect_descs_pod_ep(&podcast_episode).await;
        all_descs_pod_ep.push(desc);
        let title_pod = collect_titles_pod(&podcast_episode).await;
        all_titles_pod.push(title_pod);
        let duration = collect_durations_pod_ep(&podcast_episode).await;
        all_durations_pod_ep.push(duration);
    }
}
    // init for `Settings`
    let settings = vec!["Library".to_string(), "Per-Item Speed".to_string(), "Podcast Autoplay".to_string(), "Account".to_string(), "About".to_string(), "Update and uninstall".to_string()];

    // init for `SettingsAccount`
    let mut all_usernames: Vec<String> = Vec::new();
    let mut all_server_addresses: Vec<String> = Vec::new();
    if let Some(var_username) = database.default_usr.first() {
        all_usernames.push(var_username.clone());
    }
    if let Some(var_server_address) = database.default_usr.get(1) {
        all_server_addresses.push(var_server_address.clone());
    }

    // init variables for for scrolling into description section 
    let scroll_offset = 0;

    // Default view_state at launch
    let mut view_state = AppView::Home; // By default, Home will be the first AppView launched when the app start
    if _ids_cnt_list.is_empty() {

        view_state = AppView::Library; // If `Home` is empty (no book or podcast to continue)
    }

    // init start_vlc variables
    let is_cvlc = config.player.cvlc.clone();
    let is_cvlc_term = config.player.cvlc_term.clone();
    let mut start_vlc_program = match is_cvlc.as_str() {
        "1" => "cvlc".to_string(),
        _ => "vlc".to_string(),
    };
    if cfg!(target_os = "macos") {
        start_vlc_program = "/Applications/VLC.app/Contents/MacOS/VLC".to_string();
    }

    // Init for check_update
    let update_msg = check_update().await.unwrap_or_default();

    // Init ListeState for `Home` list (continue listening)
    let mut list_state_cnt_list = ListState::default(); // init the ListState ratatui's widget
    list_state_cnt_list.select(Some(0)); // select the first item of the list when app is launch

    // Init ListeState for `Library` list
    let mut list_state_library = ListState::default(); 
    list_state_library.select(Some(0)); 

    // Init ListeState for `SearchBook` list
    let mut list_state_search_results = ListState::default(); 
    list_state_search_results.select(Some(0)); 

    // Init ListState for `PodacastEpisode` list
    let mut list_state_pod_ep = ListState::default();
    list_state_pod_ep.select(Some(0));

    // Init ListState for `Settings` list
    let mut list_state_settings = ListState::default();
    list_state_settings.select(Some(0));

    // Init ListState for `SettingsAccount` list
    let mut list_state_settings_account = ListState::default();
    list_state_settings_account.select(Some(0));

    // Init ListState for `SettingsLibrary` list
    let mut list_state_settings_library = ListState::default();
    list_state_settings_library.select(Some(0));

    // Init ListState for `SettingsAbout` list
    let mut list_state_settings_about = ListState::default();
    list_state_settings_about.select(Some(0));

    // Init ListState for `SettingsUpdateUninstall` list
    let mut list_state_settings_update_uninstall = ListState::default();
    list_state_settings_update_uninstall.select(Some(0));

    // Init ListState for `SettingsAutoplay` list
    let mut list_state_settings_autoplay = ListState::default();
    list_state_settings_autoplay.select(Some(0));

    // Init ListState for `SettingsPerItemSpeed` list
    let mut list_state_settings_per_item_speed = ListState::default();
    list_state_settings_per_item_speed.select(Some(0));

    Ok(Self {
        database,
        id_selected_lib,
        token: Some(token),
        should_exit: false,
        list_state_cnt_list,
        list_state_library,
        list_state_search_results,
        list_state_pod_ep,
        list_state_settings,
        list_state_settings_account,
        list_state_settings_library,
        list_state_settings_about,
        list_state_settings_update_uninstall,
        list_state_settings_autoplay,
        list_state_settings_per_item_speed,
        _titles_cnt_list,
        auth_names_cnt_list,
        pub_year_cnt_list,
        duration_cnt_list,
        desc_cnt_list,
        _ids_cnt_list,
        view_state,
        library_needs_reload: false,
        is_chapter_list_expanded: false,
        titles_library,
        ids_library,
        auth_names_library,
        ids_search_book,
        search_mode,
        search_query,
        is_podcast,
        all_titles_pod_ep,
        all_ids_pod_ep,
        titles_pod_ep,
        ids_pod_ep,
        ids_pod_ep_search,
        ids_ep_cnt_list, 
        all_titles_pod_ep_search,
        titles_pod_ep_search,
        is_from_search_pod,
        ids_library_pod_search,
        all_ids_pod_ep_search,
        libraries_names,
        libraries_ids,
        media_types,
        library_name,
        media_type,
        lib_name_type,
        settings,
        all_usernames,
        all_server_addresses,
        username,
        server_address,
        server_address_pretty,
        scroll_offset,
        subtitles_pod_cnt_list,
        nums_ep_pod_cnt_list,
        seasons_pod_cnt_list,
        authors_pod_cnt_list,
        descs_pod_cnt_list,
        titles_pod_cnt_list,
        durations_pod_cnt_list,
        podcast_progress_cnt_list,
        podcast_published_at_cnt_list,
        episode_embedded_cover_ino_cnt_list,
        podcast_sort_newest_first,
        title_scroll_offset: 0,
        title_scroll_last_tick: std::time::Instant::now(),
        title_scroll_selected: None,
        published_year_library,
        desc_library,
        duration_library,
        auth_names_library_pod,
        all_subtitles_pod_ep,
        all_seasons_pod_ep,
        all_episodes_pod_ep,
        all_authors_pod_ep,
        all_descs_pod_ep,
        all_titles_pod,
        all_durations_pod_ep,
        subtitles_pod_ep,
        seasons_pod_ep,
        episodes_pod_ep,
        authors_pod_ep,
        descs_pod_ep,
        titles_pod,
        durations_pod_ep,
        subtitles_pod_ep_search,
        seasons_pod_ep_search,
        episodes_pod_ep_search,
        authors_pod_ep_search,
        descs_pod_ep_search,
        titles_pod_search,
        durations_pod_ep_search,
        all_subtitles_pod_ep_search,
        all_seasons_pod_ep_search,
        all_episodes_pod_ep_search,
        all_authors_pod_ep_search,
        all_descs_pod_ep_search,
        all_titles_pod_search,
        all_durations_pod_ep_search,
        auth_names_pod_search_book,
        auth_names_search_book,
        published_year_library_search_book,
        desc_library_search_book,
        duration_library_search_book,
        book_progress_cnt_list,
        book_progress_cnt_list_cur_time,
 //       book_progress_library,
 //       book_progress_library_cur_time,
        book_progress_search_book,
        book_progress_search_book_cur_time,
        is_cvlc,
        is_cvlc_term,
        start_vlc_program,
        config,
        changelog,
        update_msg,
        podcast_home_last_refresh: std::time::Instant::now(),
        image_picker,
        cover_protocol: None,
        cover_loaded_for_id: None,
        cover_fetch_requested: std::collections::HashSet::new(),
    })
    }

    // Re-fetches just the podcast "New & Unfinished" list if it's gotten stale, without
    // touching cursor position/selection or anything else - so an episode that just
    // finished (or a newly-published one) shows up without needing a manual refresh,
    // and without disrupting whatever the user is doing in the list.
    pub async fn refresh_podcast_home_if_stale(&mut self) -> Result<()> {
        // Finishing an episode doesn't push a signal to the main render loop (the
        // playback handler runs in a separate spawned task) - it just relies on this
        // periodic refresh to eventually notice the server-side "finished" flag and
        // drop it from the list. Kept short so that removal feels prompt.
        const STALE_AFTER: std::time::Duration = std::time::Duration::from_secs(8);

        if !self.is_podcast || self.podcast_home_last_refresh.elapsed() < STALE_AFTER {
            return Ok(());
        }

        let Some(token) = self.token.clone() else { return Ok(()) };

        // The refreshed list's composition/order can shift (an episode finishes and
        // drops out, a new one appears, published_at ties break differently) - remember
        // which episode the cursor was on so it can be re-found below, otherwise the
        // still-valid numeric index silently ends up pointing at a different episode and
        // the selection bar appears to jump around on its own every refresh.
        let selected_ep_id = self.list_state_cnt_list.selected()
            .and_then(|i| self.ids_ep_cnt_list.get(i))
            .cloned();

        let data = fetch_podcast_home_data(&token, self.server_address.clone(), &self.id_selected_lib, self.podcast_sort_newest_first, &self.username).await?;
        self._ids_cnt_list = data.ids;
        self._titles_cnt_list = data.titles;
        self.ids_ep_cnt_list = data.ids_ep;
        self.subtitles_pod_cnt_list = data.subtitles;
        self.nums_ep_pod_cnt_list = data.nums_ep;
        self.seasons_pod_cnt_list = data.seasons;
        self.authors_pod_cnt_list = data.authors;
        self.descs_pod_cnt_list = data.descs;
        self.titles_pod_cnt_list = data.titles_pod;
        self.durations_pod_cnt_list = data.durations;
        self.podcast_progress_cnt_list = data.progress;
        self.podcast_published_at_cnt_list = data.published_at;
        self.episode_embedded_cover_ino_cnt_list = data.embedded_cover_ino;
        self.podcast_home_last_refresh = std::time::Instant::now();

        if let Some(id) = selected_ep_id
            && let Some(new_pos) = self.ids_ep_cnt_list.iter().position(|i| *i == id) {
                self.list_state_cnt_list.select(Some(new_pos));
        } else if let Some(session) = get_listening_session().ok().flatten()
            && let Some(new_pos) = self.ids_ep_cnt_list.iter().position(|i| *i == session.id_pod) {
                // The previously selected episode is gone - most commonly because it just
                // finished and Podcast Autoplay moved on to the next one, dropping it out of
                // this "New & Unfinished" list. Follow the now-playing episode instead of
                // leaving the cursor on whatever numeric index it used to be, which would
                // otherwise silently point at a different episode once the list shifts.
                self.list_state_cnt_list.select(Some(new_pos));
        }

        Ok(())
    }

    // Applies a permutation to every one of the podcast Home list's parallel arrays at
    // once, so they can never end up desynced from each other - used both when pinning
    // the now-playing episode to the top and when toggling sort order.
    pub fn reorder_podcast_lists(&mut self, order: &[usize]) {
        self._titles_cnt_list = order.iter().map(|&i| self._titles_cnt_list[i].clone()).collect();
        self._ids_cnt_list = order.iter().map(|&i| self._ids_cnt_list[i].clone()).collect();
        self.ids_ep_cnt_list = order.iter().map(|&i| self.ids_ep_cnt_list[i].clone()).collect();
        self.subtitles_pod_cnt_list = order.iter().map(|&i| self.subtitles_pod_cnt_list[i].clone()).collect();
        self.nums_ep_pod_cnt_list = order.iter().map(|&i| self.nums_ep_pod_cnt_list[i].clone()).collect();
        self.seasons_pod_cnt_list = order.iter().map(|&i| self.seasons_pod_cnt_list[i].clone()).collect();
        self.authors_pod_cnt_list = order.iter().map(|&i| self.authors_pod_cnt_list[i].clone()).collect();
        self.descs_pod_cnt_list = order.iter().map(|&i| self.descs_pod_cnt_list[i].clone()).collect();
        self.titles_pod_cnt_list = order.iter().map(|&i| self.titles_pod_cnt_list[i].clone()).collect();
        self.durations_pod_cnt_list = order.iter().map(|&i| self.durations_pod_cnt_list[i].clone()).collect();
        self.podcast_progress_cnt_list = order.iter().map(|&i| self.podcast_progress_cnt_list[i]).collect();
        self.podcast_published_at_cnt_list = order.iter().map(|&i| self.podcast_published_at_cnt_list[i]).collect();
        self.episode_embedded_cover_ino_cnt_list = order.iter().map(|&i| self.episode_embedded_cover_ino_cnt_list[i].clone()).collect();
    }


// handle key
pub fn handle_key(&mut self, key: KeyEvent) {
    // init variable for player
    let mut is_playback = true;

    if key.kind != KeyEventKind::Press {
        return;
    }


    match key.code {
        // PLAYER //
        // toggle playback/pause
        KeyCode::Char(' ') => {
            let _ = handle_key_player(" ", self.config.player.address.as_str(), self.config.player.port.as_str(), &mut is_playback, self.username.as_str());
        }
        // jump forward
        KeyCode::Char('p') => {
            let _ = handle_key_player("p", self.config.player.address.as_str(), self.config.player.port.as_str(), &mut is_playback, self.username.as_str());
        }

        // jump backward
        KeyCode::Char('u') => {
            let _ = handle_key_player("u", self.config.player.address.as_str(), self.config.player.port.as_str(), &mut is_playback, self.username.as_str());
        }

        // next chapter
        KeyCode::Char('P') => {
            let _  = handle_key_player("P", self.config.player.address.as_str(), self.config.player.port.as_str(), &mut is_playback, self.username.as_str());
        }

        // previous chapter
        KeyCode::Char('U') => {
            let _ = handle_key_player("U", self.config.player.address.as_str(), self.config.player.port.as_str(), &mut is_playback, self.username.as_str());
        }

        // speed rate up
        KeyCode::Char('O') => {
            let _ = handle_key_player("O", self.config.player.address.as_str(), self.config.player.port.as_str(), &mut is_playback, self.username.as_str()); 
        }

        // speed rate down
        KeyCode::Char('I') => {
            let _ = handle_key_player("I", self.config.player.address.as_str(), self.config.player.port.as_str(), &mut is_playback, self.username.as_str()); 
        }

        // volume up
        KeyCode::Char('o') => {
            let _ = handle_key_player("o", self.config.player.address.as_str(), self.config.player.port.as_str(), &mut is_playback, self.username.as_str()); 
        }

        // volume down
        KeyCode::Char('i') => {
            let _ = handle_key_player("i", self.config.player.address.as_str(), self.config.player.port.as_str(), &mut is_playback, self.username.as_str()); 
        }

        // shutdown VLC
        KeyCode::Char('Y') => {
            let _ = handle_key_player("Y", self.config.player.address.as_str(), self.config.player.port.as_str(), &mut is_playback, self.username.as_str()); 
        }

        // show key bindings
        KeyCode::Char('B') => {
            let value = get_is_show_key_bindings(self.username.as_str());
            if value == "0" {
            let _ = update_is_show_key_bindings("1", self.username.as_str());
            } else if value == "1" {
            let _ = update_is_show_key_bindings("0", self.username.as_str());
            }
        }

        // toggle the currently-playing book's inline chapter list in Continue Listening
        KeyCode::Char('c') => {
            if matches!(self.view_state, AppView::Home) && !self.is_podcast {
                let is_now_playing_visible = get_listening_session().ok().flatten()
                    .is_some_and(|s| self._ids_cnt_list.contains(&s.id_item));
                if is_now_playing_visible {
                    // Remember which book the cursor was on (or, if it was on a chapter
                    // row, the now-playing book those chapters belong to) so it can be
                    // re-found afterward - expanding/collapsing shifts every row below
                    // the now-playing book by however many chapter rows appear/disappear.
                    let selected_row = self.list_state_cnt_list.selected()
                        .and_then(|i| self.build_home_rows().get(i).cloned());

                    self.is_chapter_list_expanded = !self.is_chapter_list_expanded;

                    let reposition_id = match selected_row {
                        Some(HomeRow::Book(i)) => self._ids_cnt_list.get(i).cloned(),
                        Some(HomeRow::Chapter { .. }) => get_listening_session().ok().flatten().map(|s| s.id_item),
                        None => None,
                    };
                    if let Some(id) = reposition_id
                        && let Some(new_pos) = self.build_home_rows().iter().position(|row| matches!(row, HomeRow::Book(i) if self._ids_cnt_list.get(*i) == Some(&id))) {
                            self.list_state_cnt_list.select(Some(new_pos));
                    }
                }
            }
        }

        // toggle speed-adjusted (real) vs raw content time for Elapsed/Left
        KeyCode::Char('T') => {
            let value = get_is_speed_adjusted_time(self.username.as_str());
            if value == "0" {
            let _ = update_is_speed_adjusted_time("1", self.username.as_str());
            } else if value == "1" {
            let _ = update_is_speed_adjusted_time("0", self.username.as_str());
            }
        }

        // toggle newest/oldest-first sort order for the podcast New & Unfinished list.
        // Re-sorts immediately using data already in memory - no need to re-fetch.
        KeyCode::Char('D') if self.is_podcast => {
            self.podcast_sort_newest_first = !self.podcast_sort_newest_first;

            let selected_ep_id = self.list_state_cnt_list.selected()
                .and_then(|i| self.ids_ep_cnt_list.get(i))
                .cloned();

            let mut order: Vec<usize> = (0..self.podcast_published_at_cnt_list.len()).collect();
            if self.podcast_sort_newest_first {
                order.sort_by_key(|&i| std::cmp::Reverse(self.podcast_published_at_cnt_list[i]));
            } else {
                order.sort_by_key(|&i| self.podcast_published_at_cnt_list[i]);
            }
            self.reorder_podcast_lists(&order);

            if let Some(id) = selected_ep_id
                && let Some(new_pos) = self.ids_ep_cnt_list.iter().position(|i| *i == id) {
                    self.list_state_cnt_list.select(Some(new_pos));
            }
        }

        // Marks the selected podcast episode as finished server-side and removes it
        // from the New & Unfinished list immediately, rather than waiting on the next
        // periodic refresh to notice the server no longer considers it unfinished.
        // Reuses reorder_podcast_lists (built for resorting, not removal) by permuting
        // to every index except the removed one - same effect on all 13 parallel
        // arrays, no separate per-array removal logic needed.
        KeyCode::Char('F') if self.is_podcast && matches!(self.view_state, AppView::Home) => {
            if let Some(selected) = self.list_state_cnt_list.selected()
                && let Some(id_pod) = self._ids_cnt_list.get(selected).cloned()
                && let Some(ep_id) = self.ids_ep_cnt_list.get(selected).cloned() {
                    let duration = self.podcast_progress_cnt_list.get(selected).map(|&(_, duration, _)| duration).unwrap_or(0.0);
                    let token = self.token.clone();
                    let server_address = self.server_address.clone();

                    // If this episode is the one actively playing, the live playback
                    // task (handle_l_pod_home) owns its progress syncing - every ~10s
                    // it PATCHes progress/currentTime with no isFinished field at all,
                    // which would silently revert the isFinished=true set below the
                    // moment it next runs (the list item would vanish then reappear).
                    // Flip listening_session.is_finished instead and let that task
                    // notice it (polled every ~1s) and stop playback + mark finished
                    // itself exactly once, with nothing left to race it.
                    let currently_playing_session = if get_is_vlc_running(self.username.as_str()) == "1" {
                        match get_listening_session() {
                            Ok(Some(session)) if session.id_pod == ep_id => Some(session),
                            _ => None,
                        }
                    } else {
                        None
                    };

                    if let Some(session) = currently_playing_session {
                        let _ = update_is_finished("1", session.id_session.as_str());
                    } else {
                        // Marked at full duration (not whatever partial progress it was
                        // at) - "finished" should mean fully listened, matching what a
                        // natural playback completion would also land on.
                        tokio::spawn(async move {
                            if let Err(e) = update_media_progress2_pod(&id_pod, token.as_ref(), Some(duration as u32), &duration.to_string(), true, &ep_id, server_address).await {
                                log::warn!("[mark_finished] episode {ep_id}: {e}");
                            }
                        });
                    }

                    let order: Vec<usize> = (0..self._ids_cnt_list.len()).filter(|&i| i != selected).collect();
                    self.reorder_podcast_lists(&order);

                    let new_len = self._ids_cnt_list.len();
                    if new_len == 0 {
                        self.list_state_cnt_list.select(None);
                    } else if selected >= new_len {
                        self.list_state_cnt_list.select(Some(new_len - 1));
                    }
            }
        }



        // END PLAYER //

        KeyCode::Char('/') => {
            let _ = self.search_active();
        }
        KeyCode::Char('S') => {
            self.view_state = AppView::Settings;
        }
        KeyCode::Tab => {
            if self.is_from_search_pod {
                self.is_from_search_pod = false;
            }
            self.toggle_view();
        }

        KeyCode::Char('Q') | KeyCode::Esc => {

            // display message 
            let message_quit = "Exiting the application and syncing data, please hold on.";
            let mut stdout = stdout();
            let _ = pop_message(&mut stdout, 3, message_quit);

            // close and sync session before close the app
            let token = self.token.clone();  
            let server_address = self.server_address.clone();
            let username = self.username.clone();
            let player_address = self.config.player.address.clone();
            let port = self.config.player.port.clone();
            let _ = update_is_vlc_running("0", username.as_str());


            tokio::spawn(async move {
                let () = sync_session_from_database(token, server_address, username, true, "Q", player_address, port).await;
            });

        }        

        KeyCode::Char('j') | KeyCode::Down => {
            self.select_next();
            self.scroll_offset = 0; 

        }
        // scroll up into description section
        KeyCode::Char('J') => self.scroll_offset += 1,
        // go start description section
        KeyCode::Char('H') => self.scroll_offset = 0,
        KeyCode::Char('k') | KeyCode::Up => {
            self.select_previous(); 
            self.scroll_offset = 0; 
        }

        // scroll down into description section
        KeyCode::Char('K') => {
            if usize::from(self.scroll_offset) > 0 {
                self.scroll_offset -= 1;
            }
        }
        KeyCode::Char('g') | KeyCode::Home => {
            self.select_first();
            self.scroll_offset = 0; 
        }        
        KeyCode::Char('G') | KeyCode::End => {
            self.select_last();
            self.scroll_offset = 0; 
        }
        KeyCode::Char('h') => {
            // To return to a page
            match self.view_state {
                AppView::SettingsAccount => {self.view_state = AppView::Settings} 
                AppView::SettingsLibrary => {self.view_state = AppView::Settings} 
                AppView::SettingsAbout => {self.view_state = AppView::Settings}
                AppView::SettingsUpdateUninstall => {self.view_state = AppView::Settings}
                AppView::SettingsAutoplay => {self.view_state = AppView::Settings}
                AppView::SettingsPerItemSpeed => {self.view_state = AppView::Settings}
                AppView::Settings => {self.view_state = AppView::Home}
                AppView::PodcastEpisode => {
                    if self.is_from_search_pod {
                        self.view_state = AppView::SearchBook;
                    } else {
                        self.view_state = AppView::Library;
                    }
                }
                _ => {}
            }
        }        
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
            // If the chapter list is expanded under the currently-playing book, the
            // cursor may be sitting on a chapter row rather than a book row - seek the
            // already-running VLC session directly to that chapter's start instead of
            // restarting playback entirely.
            if matches!(self.view_state, AppView::Home) && !self.is_podcast
                && let Some(selected) = self.list_state_cnt_list.selected()
                && let Some(HomeRow::Chapter { chapter, .. }) = self.build_home_rows().get(selected) {
                    // Round up, not down: chapter boundaries are fractional (e.g.
                    // 9836.105873), and seeking to the truncated whole second would land
                    // just before the real boundary - long enough for one polling tick to
                    // still classify it as the previous chapter and flash the wrong marker.
                    let start = chapter.start.unwrap_or(0.0).ceil() as u32;
                    let port = self.config.player.port.clone();
                    let address_player = self.config.player.address.clone();
                    tokio::spawn(async move {
                        let _ = seek_to_absolute_time(&address_player, &port, start);
                    });
                    return;
            }

            // Clone needed because variables will be used in a spawn
            let token = self.token.clone();
            let port = self.config.player.port.clone();
            let address_player = self.config.player.address.clone();
            let server_address = self.server_address.clone();
            let username = self.username.clone();

            // Init for `Continue Listening` (AppView::Home)
            let ids_cnt_list = self._ids_cnt_list.clone();
            let selected_cnt_list = if matches!(self.view_state, AppView::Home) && !self.is_podcast && self.is_chapter_list_expanded {
                // Any selection reaching here is a book row (chapter rows already handled
                // and returned above) - resolve its real index in `_ids_cnt_list`, since
                // the flat ListState index is offset by however many chapter rows are
                // spliced in above it.
                self.list_state_cnt_list.selected().and_then(|selected| {
                    match self.build_home_rows().get(selected) {
                        Some(HomeRow::Book(i)) => Some(*i),
                        _ => None,
                    }
                })
            } else {
                self.list_state_cnt_list.selected()
            };

            // Init for `Library`
            let ids_library = self.ids_library.clone();
            let selected_library = self.list_state_library.selected();

            // Init for `Search Book`
            let ids_search_book = self.ids_search_book.clone();
            let selected_search_book = self.list_state_search_results.selected();

            // Init for `PodcastEpisode`
            if self.is_podcast {
                if let Some(index) = selected_library
                    && let Some(_id_pod) = ids_library.get(index) {
                        self.ids_pod_ep = self.all_ids_pod_ep[index].clone();
                    }
                if let Some(index) = selected_search_book {
                    // ids_library_pod_search because we need the pod id and he is given by
                    // this variable
                    if let Some(_id_pod) = self.ids_library_pod_search.get(index) {
                        //    println!("{:?}", id_pod);
                        self.ids_pod_ep_search = self.all_ids_pod_ep_search[index].clone();
                        //   println!("{:?}", all_ids_pod_ep_search_clone[index]);
                    }}
            }
            // Init for `SettingsAccount`
            let selected_account = self.list_state_settings_account.selected();

            // Init for `SettingsLibrary`
            let selected_settings_library = self.list_state_settings_library.selected();

            // init for start_vlc
            let start_vlc_program = self.start_vlc_program.clone();
            let is_cvlc_term = self.is_cvlc_term.clone();

            // Podcast Autoplay needs these to re-fetch the live "New & Unfinished"
            // queue on each transition rather than relying on a stale snapshot - see
            // handle_l_pod_home::next_autoplay_episode.
            let id_selected_lib = self.id_selected_lib.clone();
            let podcast_sort_newest_first = self.podcast_sort_newest_first;
            let initial_published_at = selected_cnt_list.and_then(|i| self.podcast_published_at_cnt_list.get(i)).copied();

            // Init message 
            let message = "Loading the media...";

            // Now, spawn the async task based on the current view state
            match self.view_state {
                AppView::Home => {
                    if self.is_podcast {
                        // init some variables
                        let _selected_pod_ep = self.list_state_pod_ep.selected();
                        let ids_ep_cnt_list = self.ids_ep_cnt_list.clone();

                        tokio::spawn(async move {
                            // close vlc 
                            let _ = quit_vlc(address_player.as_str(), port.as_str());

                            // pkill vlc
                            pkill_vlc();

                            // before open a new session, wait to close and sync previous
                            // session
                            wait_prev_session_finished(username.clone()); 

                            // pop message
                            let mut stdout = stdout();
                            let _ = pop_message(&mut stdout, 3, message);

                            // in case where the app has been disgrafully closed (crash, kill)
                            // the last listening session is closed when app is restarted
                            let () = sync_session_from_database(
                                token.clone(), 
                                server_address.clone(), 
                                username.clone(), 
                                false, 
                                "l", 
                                address_player.clone(), 
                                port.clone()).await;

                            // start the track
                            handle_l_pod_home(
                                token.as_ref(),
                                &ids_cnt_list,
                                selected_cnt_list,
                                port,
                                address_player,
                                ids_ep_cnt_list,
                                server_address,
                                start_vlc_program,
                                is_cvlc_term,
                                username,
                                id_selected_lib,
                                podcast_sort_newest_first,
                                initial_published_at,
                            ).await;
                        });
                    } else {

                        tokio::spawn(async move {

                            // close vlc 
                            let _ = quit_vlc(address_player.as_str(), port.as_str());

                            // pkill vlc
                            pkill_vlc();

                            // before open a new session, wait to close and sync previous
                            // session
                            wait_prev_session_finished(username.clone()); 

                            // pop message
                            let mut stdout = stdout();
                            let _ = pop_message(&mut stdout, 3, message);

                            // in case where the app has been disgrafully closed (crash, kill)
                            // the last listening session is closed when app is restarted
                            let () = sync_session_from_database(
                                token.clone(), 
                                server_address.clone(), 
                                username.clone(), 
                                false, 
                                "l", 
                                address_player.clone(), 
                                port.clone()).await;

                            // start the track
                            handle_l_book(
                                token.as_ref(), 
                                ids_cnt_list, 
                                selected_cnt_list, 
                                port, 
                                address_player,
                                server_address, 
                                start_vlc_program,
                                is_cvlc_term, 
                                username,
                            ).await;
                        });

                    }}
                AppView::Settings => {
                    match self.list_state_settings.selected() {
                        Some(0) => self.view_state = AppView::SettingsLibrary,
                        Some(1) => self.view_state = AppView::SettingsPerItemSpeed,
                        Some(2) => self.view_state = AppView::SettingsAutoplay,
                        Some(3) => self.view_state = AppView::SettingsAccount,
                        _ => {}
                    }
                }
                AppView::SettingsAccount => {
                    if let Some(index) = selected_account {
                        let usr_to_delete = &self.all_usernames[index];
                        let _ = delete_user(usr_to_delete.as_str());
                    }
                }
                AppView::SettingsAutoplay => {
                    if let Some(index) = self.list_state_settings_autoplay.selected() {
                        let value = if index == 0 { "1" } else { "0" };
                        let _ = update_is_podcast_autoplay(value, &self.username);
                    }
                }
                AppView::SettingsPerItemSpeed => {
                    if let Some(index) = self.list_state_settings_per_item_speed.selected() {
                        let value = if index == 0 { "1" } else { "0" };
                        let _ = update_is_per_item_speed(value, &self.username);
                    }
                }
                AppView::SettingsLibrary => {
                    if let Some(index) = selected_settings_library {
                        let new_selected_lib = &self.libraries_ids[index];
                        let _ = update_id_selected_lib(new_selected_lib, &self.username);
                        self.library_needs_reload = true;
                    }
                }
                AppView::SettingsAbout => {
                }
                AppView::SettingsUpdateUninstall => {
                }
                AppView::Library => {
                    if self.is_podcast {
                        if let Some(index) = selected_library {
                            self.titles_pod_ep = self.all_titles_pod_ep[index].clone();
                            self.subtitles_pod_ep = self.all_subtitles_pod_ep[index].clone();
                            self.seasons_pod_ep = self.all_seasons_pod_ep[index].clone();
                            self.episodes_pod_ep = self.all_episodes_pod_ep[index].clone();
                            self.authors_pod_ep = self.all_authors_pod_ep[index].clone();
                            self.descs_pod_ep = self.all_descs_pod_ep[index].clone();
                            self.titles_pod = self.all_titles_pod[index].clone();
                            self.durations_pod_ep = self.all_durations_pod_ep[index].clone();
                            self.list_state_pod_ep.select(Some(0));
                            self.view_state = AppView::PodcastEpisode;
                        }} else {

                            tokio::spawn(async move {
                                // close vlc 
                                let _ = quit_vlc(address_player.as_str(), port.as_str());

                                // pkill vlc
                                pkill_vlc();

                                // before open a new session, wait to close and sync previous
                                // session
                                wait_prev_session_finished(username.clone()); 

                                // pop message
                                let mut stdout = stdout();
                                let _ = pop_message(&mut stdout, 3, message);

                                // in case where the app has been disgrafully closed (crash, kill)
                                // the last listening session is closed when app is restarted
                                let () = sync_session_from_database(
                                    token.clone(), 
                                    server_address.clone(), 
                                    username.clone(), 
                                    false, 
                                    "l", 
                                    address_player.clone(), 
                                    port.clone()).await;

                                // start the track
                                handle_l_book(
                                    token.as_ref(), 
                                    ids_library, 
                                    selected_library, 
                                    port, 
                                    address_player,
                                    server_address, 
                                    start_vlc_program,
                                    is_cvlc_term, 
                                    username,
                                ).await;
                            });
                        }
                }
                AppView::SearchBook => {
                    if self.is_podcast {
                        self.is_from_search_pod = true;
                        if let Some(index) = selected_search_book {
                            self.titles_pod_ep_search = self.all_titles_pod_ep_search[index].clone();
                            self.subtitles_pod_ep_search = self.all_subtitles_pod_ep_search[index].clone();
                            self.seasons_pod_ep_search = self.all_seasons_pod_ep_search[index].clone();
                            self.episodes_pod_ep_search = self.all_episodes_pod_ep_search[index].clone();
                            self.authors_pod_ep_search = self.all_authors_pod_ep_search[index].clone();
                            self.descs_pod_ep_search = self.all_descs_pod_ep_search[index].clone();
                            self.titles_pod_search = self.all_titles_pod_search[index].clone();
                            self.durations_pod_ep_search = self.all_durations_pod_ep_search[index].clone();
                            self.list_state_pod_ep.select(Some(0));
                            self.view_state = AppView::PodcastEpisode;
                        }} else {   

                            tokio::spawn(async move {
                                // close vlc 
                                let _ = quit_vlc(address_player.as_str(), port.as_str());

                                // pkill vlc
                                pkill_vlc();

                                // before open a new session, wait to close and sync previous
                                // session
                                wait_prev_session_finished(username.clone()); 

                                // pop message
                                let mut stdout = stdout();
                                let _ = pop_message(&mut stdout, 3, message);

                                // in case where the app has been disgrafully closed (crash, kill)
                                // the last listening session is closed when app is restarted
                                let () = sync_session_from_database(
                                    token.clone(), 
                                    server_address.clone(), 
                                    username.clone(), 
                                    false, 
                                    "l", 
                                    address_player.clone(), 
                                    port.clone()).await;

                                // start the track
                                handle_l_book(
                                    token.as_ref(), 
                                    ids_search_book, 
                                    selected_search_book, 
                                    port, 
                                    address_player,
                                    server_address, 
                                    start_vlc_program,
                                    is_cvlc_term, 
                                    username,
                                ).await;
                            });

                        }
                }
                AppView::PodcastEpisode => {
                    if self.is_from_search_pod {
                        // we need the index of selected_search_book to feet after with
                        // ids_library_pod_search
                        if let Some(index) = selected_search_book {
                            // ids_library_pod_search because we need the pod id and he is given by
                            // this variable
                            if let Some(id_pod) = self.ids_library_pod_search.get(index) {
                                //    println!("{:?}", id_pod);
                                let all_ids_pod_ep_search_clone = self.all_ids_pod_ep_search.clone();
                                //   println!("{:?}", all_ids_pod_ep_search_clone[index]);
                                let id_pod_clone = id_pod.clone();
                                let selected_pod_ep = self.list_state_pod_ep.selected();

                                tokio::spawn(async move {
                                    // close vlc 
                                    let _ = quit_vlc(address_player.as_str(), port.as_str());

                                    // pkill vlc
                                    pkill_vlc();

                                    // before open a new session, wait to close and sync previous
                                    // session
                                    wait_prev_session_finished(username.clone()); 

                                    // pop message
                                    let mut stdout = stdout();
                                    let _ = pop_message(&mut stdout, 3, message);

                                    // in case where the app has been disgrafully closed (crash, kill)
                                    // the last listening session is closed when app is restarted
                                    let () = sync_session_from_database(
                                        token.clone(), 
                                        server_address.clone(), 
                                        username.clone(), 
                                        false, 
                                        "l", 
                                        address_player.clone(), 
                                        port.clone()).await;

                                    // start the track
                                    handle_l_pod(
                                        token.as_ref(), 
                                        &all_ids_pod_ep_search_clone[index], 
                                        selected_pod_ep, 
                                        port, 
                                        address_player,
                                        id_pod_clone.as_str(), 
                                        server_address, 
                                        start_vlc_program,
                                        is_cvlc_term, 
                                        username,
                                    ).await;
                                });
                            }
                        }
                    } else {
                        // selected_livrary ids_library because we need the pod id and he is given by
                        // these variables
                        // we also need the index of selected library to feet after with
                        // ids_library
                        if let Some(index) = selected_library
                            && let Some(id_pod) = ids_library.get(index) {
                                let all_ids_pod_ep_clone = self.all_ids_pod_ep.clone();
                                self.ids_pod_ep = all_ids_pod_ep_clone[index].clone();
                                let id_pod_clone = id_pod.clone();
                                let selected_pod_ep = self.list_state_pod_ep.selected();
                                tokio::spawn(async move {
                                    // close vlc 
                                    let _ = quit_vlc(address_player.as_str(), port.as_str());

                                    // pkill vlc
                                    pkill_vlc();

                                    // before open a new session, wait to close and sync previous
                                    // session
                                    wait_prev_session_finished(username.clone()); 

                                    // pop message
                                    let mut stdout = stdout();
                                    let _ = pop_message(&mut stdout, 3, message);

                                    // in case where the app has been disgrafully closed (crash, kill)
                                    // the last listening session is closed when app is restarted
                                    let () = sync_session_from_database(
                                        token.clone(), 
                                        server_address.clone(), 
                                        username.clone(), 
                                        false, 
                                        "l", 
                                        address_player.clone(), 
                                        port.clone()).await;

                                    // start the track
                                    handle_l_pod(
                                        token.as_ref(), 
                                        &all_ids_pod_ep_clone[index], 
                                        selected_pod_ep, 
                                        port, 
                                        address_player,
                                        id_pod_clone.as_str(), 
                                        server_address, 
                                        start_vlc_program,
                                        is_cvlc_term, 
                                        username,
                                    ).await;
                                });
                            }

                    }
                }
            }
        }
        _ => {}
    }
}


/// Toggle between Home and Library views
fn toggle_view(&mut self) {
    self.view_state = match self.view_state {
        AppView::Home => AppView::Library,
        AppView::Library => AppView::Home,
        AppView::SearchBook => AppView::Home,
        AppView::PodcastEpisode => AppView::Home,
        AppView::Settings => AppView::Home,
        AppView::SettingsAccount => AppView::Home,
        AppView::SettingsLibrary => AppView::Home,
        AppView::SettingsAbout => AppView::Home,
        AppView::SettingsUpdateUninstall => AppView::Home,
        AppView::SettingsAutoplay => AppView::Home,
        AppView::SettingsPerItemSpeed => AppView::Home,

    };
}

/// Flattens the Continue Listening list into individual rows, splicing indented chapter
/// rows in directly beneath the currently-playing book's row when `is_chapter_list_expanded`
/// is set. Returns plain `Book` rows 1:1 with `_ids_cnt_list` for podcasts, or whenever
/// nothing is expanded - so callers never need to special-case those situations themselves.
pub fn build_home_rows(&self) -> Vec<HomeRow> {
    if self.is_podcast || !self.is_chapter_list_expanded {
        return (0..self._ids_cnt_list.len()).map(HomeRow::Book).collect();
    }

    let active_session = get_listening_session().ok().flatten();
    let chapters: Vec<Chapter> = active_session.as_ref()
        .map(|s| serde_json::from_str(&s.chapters).unwrap_or_default())
        .unwrap_or_default();

    let mut rows = Vec::new();
    for i in 0..self._ids_cnt_list.len() {
        rows.push(HomeRow::Book(i));

        let is_now_playing = active_session.as_ref()
            .is_some_and(|s| self._ids_cnt_list.get(i) == Some(&s.id_item));
        if is_now_playing {
            for chapter in &chapters {
                rows.push(HomeRow::Chapter { book_index: i, chapter: chapter.clone() });
            }
        }
    }

    rows
}

/// Select functions that apply to both views
/// all select functions are from `ListState` widget
pub fn select_next(&mut self) {
    match self.view_state {
        AppView::Home => { if let Some(selected) = self.list_state_cnt_list.selected() {
            if selected + 1  < self.build_home_rows().len() {
                self.list_state_cnt_list.select_next();
            } else {
                self.list_state_cnt_list.select_first();
            }}}
        AppView::Library => { if let Some(selected) = self.list_state_library.selected() {
            if selected + 1  < self.ids_library.len() {
                self.list_state_library.select_next();
            } else {
                self.list_state_library.select_first();
            }}}
        AppView::SearchBook => { if let Some(selected) = self.list_state_search_results.selected() {
            if selected + 1  < self.ids_search_book.len() {
                self.list_state_search_results.select_next();
            } else {
                self.list_state_search_results.select_first();
            }}}
        AppView::PodcastEpisode => { if let Some(selected) = self.list_state_pod_ep.selected() {
            if self.is_from_search_pod {
                if selected + 1  < self.ids_pod_ep_search.len() {
                    self.list_state_pod_ep.select_next();
                } else {
                    self.list_state_pod_ep.select_first();
                }
            } else {
                if selected + 1  < self.ids_pod_ep.len() {
                    self.list_state_pod_ep.select_next();
                } else {
                    self.list_state_pod_ep.select_first();
                }}}}
        AppView::Settings => { if let Some(selected) = self.list_state_settings.selected() {
            if selected + 1  < self.settings.len() {
                self.list_state_settings.select_next();
            } else {
                self.list_state_settings.select_first();
            }}}
        AppView::SettingsAccount => self.list_state_settings_account.select_next(),
        AppView::SettingsLibrary => { if let Some(selected) = self.list_state_settings_library.selected() {
            if selected + 1  < self.media_types.len() {
                self.list_state_settings_library.select_next();
            } else {
                self.list_state_settings_library.select_first();
            }}}
        AppView::SettingsAbout => self.list_state_settings_about.select_next(),
        AppView::SettingsUpdateUninstall => self.list_state_settings_update_uninstall.select_next(),
        AppView::SettingsAutoplay => { if let Some(selected) = self.list_state_settings_autoplay.selected() {
            if selected + 1 < 2 {
                self.list_state_settings_autoplay.select_next();
            } else {
                self.list_state_settings_autoplay.select_first();
            }}}
        AppView::SettingsPerItemSpeed => { if let Some(selected) = self.list_state_settings_per_item_speed.selected() {
            if selected + 1 < 2 {
                self.list_state_settings_per_item_speed.select_next();
            } else {
                self.list_state_settings_per_item_speed.select_first();
            }}}
    }
}

pub fn select_previous(&mut self) {
    match self.view_state {
        AppView::Home => self.list_state_cnt_list.select_previous(),
        AppView::Library => self.list_state_library.select_previous(),
        AppView::SearchBook => self.list_state_search_results.select_previous(),
        AppView::PodcastEpisode => self.list_state_pod_ep.select_previous(),
        AppView::Settings => self.list_state_settings.select_previous(),
        AppView::SettingsAccount => self.list_state_settings_account.select_previous(),
        AppView::SettingsLibrary => self.list_state_settings_library.select_previous(),
        AppView::SettingsAbout => self.list_state_settings_about.select_previous(),
        AppView::SettingsUpdateUninstall => self.list_state_settings_update_uninstall.select_previous(),
        AppView::SettingsAutoplay => self.list_state_settings_autoplay.select_previous(),
        AppView::SettingsPerItemSpeed => self.list_state_settings_per_item_speed.select_previous(),
    }
}

pub fn select_first(&mut self) {
    match self.view_state {
        AppView::Home => self.list_state_cnt_list.select_first(),
        AppView::Library => self.list_state_library.select_first(),
        AppView::SearchBook => self.list_state_search_results.select_first(),
        AppView::PodcastEpisode => self.list_state_pod_ep.select_first(),
        AppView::Settings => self.list_state_settings.select_first(),
        AppView::SettingsAccount => self.list_state_settings_account.select_first(),
        AppView::SettingsLibrary => self.list_state_settings_library.select_first(),
        AppView::SettingsAbout => self.list_state_settings_about.select_first(),
        AppView::SettingsUpdateUninstall => self.list_state_settings_update_uninstall.select_first(),
        AppView::SettingsAutoplay => self.list_state_settings_autoplay.select_first(),
        AppView::SettingsPerItemSpeed => self.list_state_settings_per_item_speed.select_first(),
    }
}

pub fn select_last(&mut self) {
    match self.view_state {
        AppView::Home => {
            let last_index = self.build_home_rows().len() - 1;
            self.list_state_cnt_list.select(Some(last_index));
        }
        AppView::Library => {
            let last_index = self.ids_library.len() - 1;
            self.list_state_library.select(Some(last_index));
        }            
        AppView::SearchBook => {
            let last_index = self.ids_search_book.len() - 1;
            self.list_state_search_results.select(Some(last_index));
        }            
        AppView::PodcastEpisode => {
            if self.is_from_search_pod {
                let last_index = self.ids_pod_ep_search.len() - 1;
                self.list_state_pod_ep.select(Some(last_index));
            } else {
                let last_index = self.ids_pod_ep.len() - 1;
                self.list_state_pod_ep.select(Some(last_index));
            }}            
        AppView::Settings => {
            let last_index = self.settings.len() - 1;
            self.list_state_settings.select(Some(last_index));
        }            
        AppView::SettingsAccount => self.list_state_settings_account.select_last(),
        AppView::SettingsLibrary => {
            let last_index = self.media_types.len() - 1;
            self.list_state_settings_library.select(Some(last_index));
        }            
        AppView::SettingsAbout => self.list_state_settings_about.select_last(),
        AppView::SettingsUpdateUninstall => self.list_state_settings_update_uninstall.select_last(),
        AppView::SettingsAutoplay => self.list_state_settings_autoplay.select(Some(1)),
        AppView::SettingsPerItemSpeed => self.list_state_settings_per_item_speed.select(Some(1)),
    }
}

}
