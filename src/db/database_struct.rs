use serde::{Serialize, Deserialize};
use crate::db::crud::{init_db, select_default_usr, update_login_err};
use color_eyre::Result;
use log::error;
use std::time::Duration;

pub struct Database  {
    pub users: Vec<User>,
    pub default_usr: Vec<String>,
    pub listening_session: ListeningSession,
    pub others: Others,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub  server_address: String,
    pub  username: String,
    pub  token: String,
    pub  is_default_usr: bool,
    pub  name_selected_lib: String,
    pub  id_selected_lib: String,
    pub  is_loop_break: String,
    pub  is_vlc_launched_first_time: String,
    pub  speed_rate: f32,
    pub  is_vlc_running: String,
    pub  is_show_key_bindings: String,
    pub  is_speed_adjusted_time: String,
    pub  is_podcast_autoplay: String,
    pub  is_per_item_speed: String,
}

#[derive(Serialize, Deserialize, Debug)]
// currently use for close listening session when app is quit
// but in future could be used to sync offline items
pub struct ListeningSession {
    pub id_session: String,
    pub id_item: String,
    pub current_time: u32,
    pub duration: String,
    pub is_finished: bool,
    pub id_pod: String,
    pub elapsed_time: u32,
    pub title: String,
    pub author: String,
    pub is_playback: bool,
    pub chapter: String,
    pub chapters: String,
    pub volume: i32,
}

pub struct Others {
    pub login_err: String,
}


impl Database {
    pub async fn new() -> Result<Self> {
        // open db and create table if there is none
        let _ = init_db();

        // init empty Vec<User> for future add of users
        let users: Vec<User> = vec![];

        // Retrieve the default user. A failed query here (eg. the db is briefly
        // locked right after a user was deleted/changed and the app was immediately
        // restarted) used to be silently treated the same as "no default user
        // exists", sending a returning user back to the login screen with no
        // explanation (bug_id 2eb9e3). Retry a few times first since this is
        // normally transient and self-resolves within a second or two; if it's
        // still failing after that, surface it via the login screen's existing
        // error banner instead of staying silent.
        let mut default_usr: Vec<String> = Vec::new();
        let mut last_err = None;
        for attempt in 0..5 {
            match select_default_usr() {
                Ok(result) => {
                    default_usr = result;
                    last_err = None;
                    break;
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt < 4 {
                        tokio::time::sleep(Duration::from_millis(400)).await;
                    }
                }
            }
        }
        if let Some(e) = last_err {
            let message = format!("Couldn't read your saved login ({e}) - please log in again.");
            error!("[Database::new] {message}");
            let _ = update_login_err(&message);
        }


        // init listening_session
        let listening_session = ListeningSession {
            id_session: String::new(),
            id_item: String::new(),
            current_time: 0,
            duration: String::new(),
            is_finished: false,
            id_pod: String::new(),
            elapsed_time: 0,
            title: String::new(),
            author: String::new(),
            is_playback: false,
            chapter: String::new(),
            chapters: String::new(),
            volume: 100,
        };

        let others = Others {
            login_err: String::new(),
        };

        Ok(Self {
            users,
            default_usr,
            listening_session,
            others
        })
    }
}

