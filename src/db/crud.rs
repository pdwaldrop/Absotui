use rusqlite::{params, Connection, Result};
use crate::db::database_struct::User;
use crate::db::database_struct::ListeningSession;
use crate::db::database_struct::Others;
use crate::utils::pop_up_message::pop_message;
use std::io::stdout;
use log::{info, error};
use std::env;
use std::path::PathBuf;


// Update is_show_key_bindings
pub fn update_is_show_key_bindings(value: &str, username: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");


    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE users SET is_show_key_bindings = ?1 WHERE username = ?2",
            params![value, username],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_is_show_key_bindings] {err_message}");
    }

    Ok(())
}


// get is_show_key_bindings
pub fn get_is_show_key_bindings(username: &str) -> String {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(_) => return String::from("Error: unable open database"),
    };

    let mut stmt = match conn.prepare("SELECT is_show_key_bindings FROM users WHERE username = ?1") {
        Ok(s) => s,
        Err(_) => return String::from("Error to prepare reqwest"),
    };

    match stmt.query_row(params![username], |row| row.get::<_, String>(0)) {
        Ok(id) => id.clone(),
        Err(_) => String::from("No db found"),
    }
}

// Update is_speed_adjusted_time
pub fn update_is_speed_adjusted_time(value: &str, username: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");


    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE users SET is_speed_adjusted_time = ?1 WHERE username = ?2",
            params![value, username],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_is_speed_adjusted_time] {err_message}");
    }

    Ok(())
}


// get is_speed_adjusted_time
pub fn get_is_speed_adjusted_time(username: &str) -> String {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(_) => return String::from("Error: unable open database"),
    };

    let mut stmt = match conn.prepare("SELECT is_speed_adjusted_time FROM users WHERE username = ?1") {
        Ok(s) => s,
        Err(_) => return String::from("Error to prepare reqwest"),
    };

    match stmt.query_row(params![username], |row| row.get::<_, String>(0)) {
        Ok(id) => id.clone(),
        Err(_) => String::from("No db found"),
    }
}

// Update is_vlc_running
pub fn update_is_vlc_running(value: &str, username: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE users SET is_vlc_running = ?1 WHERE username = ?2",
            params![value, username],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_is_vlc_running] {err_message}");
    }

    Ok(())
}


// get is_vlc_running
pub fn get_is_vlc_running(username: &str) -> String {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(_) => return String::from("Error: unable open database"),
    };

    let mut stmt = match conn.prepare("SELECT is_vlc_running FROM users WHERE username = ?1") {
        Ok(s) => s,
        Err(_) => return String::from("Error to prepare reqwest"),
    };

    match stmt.query_row(params![username], |row| row.get::<_, String>(0)) {
        Ok(id) => id.clone(),
        Err(_) => String::from("No db found"),
    }
}

// Update speed_rate
pub fn update_speed_rate(username: &str, is_speed_rate_up: bool) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        if is_speed_rate_up {
        conn.execute(
            "UPDATE users SET speed_rate = speed_rate + 0.10 WHERE username = ?1",
            params![username],
        )?;
        } else {
        conn.execute(
            "UPDATE users SET speed_rate = speed_rate - 0.10 WHERE username = ?1",
            params![username],
        )?;
        }
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_speed_rate] {err_message}");
    }

    Ok(())
}


// get speed_rate
pub fn get_speed_rate(username: &str) -> String {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(_) => return String::from("Error: unable open database"),
    };

    let mut stmt = match conn.prepare("SELECT speed_rate FROM users WHERE username = ?1") {
        Ok(s) => s,
        Err(_) => return String::from("Error to prepare reqwest"),
    };

    match stmt.query_row(params![username], |row| row.get::<_, f32>(0)) {
        Ok(id) => id.to_string(),
        Err(_) => String::from("No db found"),
    }
}

// get listening_session
pub fn get_listening_session() -> Result<Option<ListeningSession>> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {
        let mut stmt = conn.prepare(
            "SELECT id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter
             FROM listening_session
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![])?;

        if let Some(row) = rows.next()? {
            let session = ListeningSession {
                id_session: row.get(0)?,
                id_item: row.get(1)?,
                current_time: row.get(2)?,
                duration: row.get(3)?,
                is_finished: row.get(4)?,
                id_pod: row.get(5)?,
                elapsed_time: row.get(6)?,
                title: row.get(7)?,
                author: row.get(8)?,
                is_playback: row.get(9)?,
                chapter: row.get(10)?,
            };
            return Ok(Some(session));
        }
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[get_listening_session] {err_message}");
    }

    Ok(None)
}

// insert data into `listening_session` table
pub fn insert_listening_session(
    id_session: String,
    id_item: String,
    current_time: u32,
    duration: String,
    id_pod: String,
    elapsed_time: u32,
    title: String,
    author: String,
    is_playback: bool,
    chapter: String,

) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {
        conn.execute("DELETE FROM listening_session", params![])?;
        conn.execute(
            "INSERT INTO listening_session (id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter) 
             VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![id_session, id_item, current_time, duration, id_pod, elapsed_time, title, author, is_playback, chapter],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[insert_listening_session] {err_message}");
    }

    Ok(())
}

// Update chapter (for `listening_session` table)
pub fn update_chapter(value: &str, id_session: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE listening_session SET chapter = ?1 WHERE id_session = ?2",
            params![value, id_session],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_chapter] {err_message}");
    }

    Ok(())
}
// Update is_playback (for `listening_session` table)
pub fn update_is_playback(value: &str, id_session: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE listening_session SET is_playback = ?1 WHERE id_session = ?2",
            params![value, id_session],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_is_playback] {err_message}");
    }

    Ok(())
}
// Update current_time (for `listening_session` table)
pub fn update_current_time(value: u32, id_session: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE listening_session SET current_time_playback = ?1 WHERE id_session = ?2",
            params![value, id_session],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_current_time] {err_message}");
    }

    Ok(())
}

// Update elapsed_time (for `listening_session` table)
pub fn update_elapsed_time(value: u32, id_session: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE listening_session SET elapsed_time = elapsed_time + ?1 WHERE id_session = ?2",
            params![value, id_session],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_elapsed_time] {err_message}");
    }

    Ok(())
}

// Update is_finished (for `listening_session` table)
pub fn update_is_finished(value: &str, id_session: &str) -> Result<()> {
    
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE listening_session SET is_finished = ?1 WHERE id_session = ?2",
            params![value, id_session],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_is_finished] {err_message}");
    }

    Ok(())
}

// Delete an user
pub fn delete_user(username: &str) -> Result<()> {
    
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let message = format!("User '{username}' deleted. Please restart the app to apply the changes.");
    let err_message = "Error connecting to the database.";
    if let Ok(conn) = Connection::open(db_path) {

        let rows_deleted = conn.execute(
            "DELETE FROM users WHERE username = ?1",
            params![username],
        )?;

        if rows_deleted > 0 {
            let mut stdout = stdout();
            let _ = pop_message(&mut stdout, 3, message.as_str());
            info!("[delete_user] User deleted.");
        } else {
            //println!("No user found with this username '{}'.", username);
        }
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[delete user] {err_message}");
    }

    Ok(())
}

// Update is_loop_break
pub fn update_is_loop_break(value: &str, username: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE users SET is_loop_break = ?1 WHERE username = ?2",
            params![value, username],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_is_loop_break] {err_message}");
    }

    Ok(())
}


// get is_loop_break
pub fn get_is_loop_break(username: &str) -> String {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(_) => return String::from("Error: unable open database"),
    };

    let mut stmt = match conn.prepare("SELECT is_loop_break FROM users WHERE username = ?1") {
        Ok(s) => s,
        Err(_) => return String::from("Error to prepare reqwest"),
    };

    match stmt.query_row(params![username], |row| row.get::<_, String>(0)) {
        Ok(id) => id,
        Err(_) => String::from("No db found"),
    }
}

// Update is_vlv_launched_first_time
pub fn update_is_vlc_launched_first_time(value: &str, username: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE users SET is_vlc_launched_first_time = ?1 WHERE username = ?2",
            params![value, username],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[is_vlc_launched_first_time] {err_message}");
    }

    Ok(())
}
// get is_vlc_launched_first_time
pub fn get_is_vlc_launched_first_time(username: &str) -> String {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(_) => return String::from("Error: unable open database"),
    };

    let mut stmt = match conn.prepare("SELECT is_vlc_launched_first_time FROM users WHERE username = ?1") {
        Ok(s) => s,
        Err(_) => return String::from("Error to prepare reqwest"),
    };

    match stmt.query_row(params![username], |row| row.get::<_, String>(0)) {
        Ok(id) => id,
        Err(_) => String::from("No db found"),
    }
}
// Update id_selected_lib
pub fn update_id_selected_lib(id_selected_lib: &str, username: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let message = "The library has been updated. Please refresh the app to apply the changes.";
    let err_message = "Error connecting to the database.";
    if let Ok(conn) = Connection::open(db_path) {

        conn.execute(
            "UPDATE users SET id_selected_lib = ?1 WHERE username = ?2",
            params![id_selected_lib, username],
        )?;
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, message);
        info!("[update_id_selected_lib] The library has been updated");

    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_id_selected_lib] {err_message}");
    }

    Ok(())
}

// update default user 
//pub fn update_default_user(conn: &Connection, username: &str) -> Result<()> {
//    // Mark all user as 0 by default
//    conn.execute(
//        "UPDATE users SET is_default_usr = 0",
//        [],
//    )?;
//
//    // Put the desired user as default
//    conn.execute(
//        "UPDATE users SET is_default_usr = 1 WHERE username = ?1",
//        params![username],
//    )?;
//
//    Ok(())
//}

// Insert user in database
pub fn db_insert_usr(users : &Vec<User>)  -> Result<()> {   
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let conn = Connection::open(db_path)?;
    for user in users {
        conn.execute(
            "INSERT OR REPLACE INTO users (username, server_address, token, is_default_usr, name_selected_lib, id_selected_lib, is_loop_break, is_vlc_launched_first_time, speed_rate, is_vlc_running, is_show_key_bindings, is_speed_adjusted_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
            user.username,
            user.server_address,
            user.token,
            i32::from(user.is_default_usr),
            user.name_selected_lib,
            user.id_selected_lib,
            user.is_loop_break,
            user.is_vlc_launched_first_time,
            user.speed_rate,
            user.is_vlc_running,
            user.is_show_key_bindings,
            user.is_speed_adjusted_time,
            ],
        )?;
    }
    Ok(())
}

// get others
pub fn get_others() -> Result<Option<Others>> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {
        let mut stmt = conn.prepare(
            "SELECT login_err
             FROM others
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![])?;

        if let Some(row) = rows.next()? {
            let others = Others {
                login_err: row.get(0)?,
            };
            return Ok(Some(others));
        }
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[get_others] {err_message}");
    }

    Ok(None)
}
// Update login_err (for `others` table)
pub fn update_login_err(value: &str) -> Result<()> {

    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let err_message = "Error connecting to the database.";

    if let Ok(conn) = Connection::open(db_path) {
        conn.execute(
            "INSERT INTO others (login_err) SELECT '' WHERE NOT EXISTS (SELECT 1 FROM others LIMIT 1)",
            [],
        )?;
        conn.execute(
            "UPDATE others SET login_err = ?1 WHERE rowid = 1",
            params![value],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_login_err] {err_message}");
    }

    Ok(())
}

// Select default user
pub fn select_default_usr() -> Result<Vec<String>> {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT username, server_address, token, is_default_usr, name_selected_lib, id_selected_lib, is_loop_break, is_vlc_launched_first_time, speed_rate, is_vlc_running, is_show_key_bindings, is_speed_adjusted_time
         FROM users WHERE is_default_usr = 1 LIMIT 1"
    )?;


    let user_iter = stmt.query_map([], |row| {
        Ok(User {
            username: row.get(0)?,
            server_address: row.get(1)?,
            token: row.get(2)?,
            is_default_usr: row.get::<_, i32>(3)? != 0,  // convert 0/1 in bool
            name_selected_lib: row.get(4)?,
            id_selected_lib: row.get(5)?,
            is_loop_break: row.get(6)?,
            is_vlc_launched_first_time: row.get(7)?,
            speed_rate: row.get(8)?,
            is_vlc_running: row.get(9)?,
            is_show_key_bindings: row.get(10)?,
            is_speed_adjusted_time: row.get(11)?,
        })
    })?;

    let mut result = Vec::new();

    for user in user_iter {
        match user {
            Ok(user) => {
                result.push(user.username);
                result.push(user.server_address);
                result.push(user.token);
                result.push(user.is_default_usr.to_string());
                result.push(user.name_selected_lib);
                result.push(user.id_selected_lib);
                result.push(user.is_loop_break);
                result.push(user.is_vlc_launched_first_time);
                result.push(user.speed_rate.to_string());
                result.push(user.is_vlc_running);
                result.push(user.is_show_key_bindings);
                result.push(user.is_speed_adjusted_time);
            }
            Err(e) => {
                println!("Error occurred: {e}");
                //return Err(rusqlite::Error::FromSqlConversionFailure(0, "Failed to map user".to_string()));
            }
        }
    }

    if result.is_empty() {
        //println!("No default user found.");
    }

    Ok(result)  
}

// Init db and table if not exist
pub fn init_db() -> Result<()> {
    let config_home_path = env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");

            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }

            path
        }, PathBuf::from);

    let db_path = config_home_path.join("absotui/db.sqlite3");

    // Open or create db
    let conn = Connection::open(db_path)?;

    //Create table `users` if there is none 
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
                username TEXT PRIMARY KEY,
                server_address TEXT NOT NULL,
                token TEXT NOT NULL,
                is_default_usr INTEGER NOT NULL DEFAULT 0,
                name_selected_lib TEXT NOT NULL,
                id_selected_lib TEXT NOT NULL,
                is_loop_break TEXT NOT NULL,
                is_vlc_launched_first_time TEXT NOT NULL,
                speed_rate FLOAT NOT NULL,
                is_vlc_running TEXT NOT NULL,
                is_show_key_bindings TEXT NOT NULL,
                is_speed_adjusted_time TEXT NOT NULL DEFAULT '1'
            )",
        [],
    )?;

    // Migration for databases created before `is_speed_adjusted_time` existed.
    // SQLite has no "ADD COLUMN IF NOT EXISTS", so we just ignore the error
    // when the column is already there.
    let _ = conn.execute(
        "ALTER TABLE users ADD COLUMN is_speed_adjusted_time TEXT NOT NULL DEFAULT '1'",
        [],
    );

    //Create table `listening_session` if there is none 
    conn.execute(
        "CREATE TABLE IF NOT EXISTS listening_session (
            id_session TEXT PRIMARY KEY,
            id_item TEXT NOT NULL,
            current_time_playback INTEGER NOT NULL,
            duration TEXT NOT NULL,
            is_finished INTEGER NOT NULL DEFAULT 0,
            id_pod TEXT NOT NULL,
            elapsed_time INTEGER NOT NULL,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            is_playback INTEGER NOT NULL DEFAULT 1,
            chapter TEXT NOT NULL
            )",
        [],
    )?;

    //Create table `others` if there is none 
    conn.execute(
        "CREATE TABLE IF NOT EXISTS others (
            login_err TEXT NOT NULL DEFAULT ''
        )",
        [],
    )?;

    Ok(())
}


