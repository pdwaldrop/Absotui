use log::info;
use crate::db::crud::{get_listening_session, get_speed_rate, get_is_speed_adjusted_time};

pub fn player_info(username: &str) -> Vec<String> {
    let mut player_info = Vec::new();
    let is_speed_adjusted_time = get_is_speed_adjusted_time(username) == "1";

    match get_listening_session() {
        Ok(Some(session)) => {
            player_info.push(session.title);
            player_info.push(session.author);

            // Podcast episodes are single audio files with no chapters - VLC still
            // reports a "chapter" for them (always 0, i.e. "Chapter 1"), so that field
            // is only meaningful for books. id_pod is only set for podcast sessions.
            if !session.id_pod.is_empty() {
                player_info.push(String::new());
            } else if let Ok(num) = session.chapter.trim().parse::<u32>() {
                let new_chapter = format!("Chapter {}", num + 1);
                player_info.push(new_chapter);
            } else {
                player_info.push(session.chapter.clone());
            }

            player_info.push(session.is_playback.to_string());
            player_info.push(format_time(session.current_time));

            // `session.current_time` comes straight from VLC's real, unscaled position in
            // the content, so the total duration it's compared against must stay unscaled
            // too - dividing only the duration by speed_rate here used to desync the two,
            // making progress hit 100% long before the content actually finished.
            let original_duration = session.duration.parse::<u32>().unwrap_or(0);
            player_info.push(format_time(original_duration));

            let speed_rate: f32 = get_speed_rate(username).parse().unwrap_or(1.0);
            let remaining_time = original_duration.saturating_sub(session.current_time);

            // Session (time so far this playback session) and Remaining are the two fields
            // that can mean either "real, speed-adjusted time" or "raw content time".
            // Current/Duration/Percent always stay in raw content-time, since your position
            // in the story doesn't change depending on how fast you're listening to it.
            if is_speed_adjusted_time {
                player_info.push(format_time(session.elapsed_time));
                let remaining_time_adjusted = (remaining_time as f32 / speed_rate) as u32;
                player_info.push(format_time(remaining_time_adjusted));
            } else {
                let elapsed_content_equivalent = (session.elapsed_time as f32 * speed_rate) as u32;
                player_info.push(format_time(elapsed_content_equivalent));
                player_info.push(format_time(remaining_time));
            }

            let percent_progress = (session.current_time as f32 / original_duration as f32) * 100.0;
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
    player_info.push(if is_speed_adjusted_time { "Real".to_string() } else { "Content".to_string() });

    player_info
}

pub fn format_time(seconds: u32) -> String {
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
