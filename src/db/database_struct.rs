use serde::{Serialize, Deserialize};
use crate::db::crud::{init_db, select_default_usr};
use color_eyre::Result;

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

        // retrieve default user
        let mut default_usr: Vec<String> = Vec::new();

        if let Ok(result) = select_default_usr() {
            default_usr = result;
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

