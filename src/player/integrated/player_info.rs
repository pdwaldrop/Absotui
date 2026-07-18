use log::info;
use crate::db::crud::{get_listening_session, get_speed_rate};

pub fn player_info(username: &str) -> Vec<String> {
    let mut player_info = Vec::new();

    match get_listening_session() {
        Ok(Some(session)) => {
            player_info.push(session.title);
            player_info.push(session.author);

            if let Ok(num) = session.chapter.trim().parse::<u32>() {
                let new_chapter = format!("Chapter {}", num + 1);
                player_info.push(new_chapter);
            } else {
                player_info.push(session.chapter.clone()); 
            }

            player_info.push(session.is_playback.to_string());
            player_info.push(format_time(session.current_time));

            let speed_rate_str = get_speed_rate(username);
            let speed_rate: f32 = speed_rate_str.parse().unwrap_or(1.0);
            let original_duration = session.duration.parse::<u32>().unwrap_or(0);
            let adjusted_duration = (original_duration as f32 / speed_rate) as u32;
            player_info.push(format_time(adjusted_duration)); 

            let remaining_time = adjusted_duration.saturating_sub(session.current_time);
            player_info.push(format_time(session.elapsed_time));
            player_info.push(format_time(remaining_time)); 

            let percent_progress = (session.current_time as f32 / adjusted_duration as f32) * 100.0;
            player_info.push(format!("{}", percent_progress.round() as u32));
        }
        Ok(None) => {
            player_info.push("N/A".to_string());
        }
        Err(e) => {
            player_info.push("Error".to_string());
            info!("[player_info] Error retrieving data: {e}");
        }
    }

    player_info.push(get_speed_rate(username));

    player_info
}

fn format_time(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{secs:02}")
    } else if minutes > 0 {
        format!("{minutes}:{secs:02}")
    } else {
        format!("0:{secs}")
    }
}
