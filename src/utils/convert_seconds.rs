pub fn convert_seconds(vec_seconds: Vec<f64>) -> Vec<String> {
    vec_seconds.iter()
        .map(|&s| {
            let total_minutes = (s / 60.0).round() as i64;
            let hours = total_minutes / 60;
            let minutes = total_minutes % 60;

            if hours == 0 {
                format!("{minutes}m")
            } else if minutes == 0 {
                format!("{hours}h")
            } else {
                format!("{hours}h{minutes}m")
            }
        })
        .collect()
}


pub fn convert_seconds_for_prg(duration: f64, current_time: f64) -> String {
            let time_left_s = duration - current_time;
            let total_minutes = (time_left_s / 60.0).round() as i64;
            let hours = total_minutes / 60;
            let minutes = total_minutes % 60;

            if current_time == 0.0 {
                String::new()
            }
            else if hours == 0 {
                format!("{minutes}m left,")
            } else if minutes == 0 {
                format!("{hours}h left,")
            } else {
                format!("{hours}h{minutes}m left,")
            }
        }

/// Absolute difference between two playback positions, in whole seconds. Used to detect
/// a large jump (chapter skip, `p`/`u` +/-10s) between polls of VLC's position. Takes the
/// difference as f64 first - jumping backward makes `new_time` smaller than `old_time`,
/// and subtracting as u32 directly would underflow and panic.
pub fn progress_time_diff(new_time: f64, old_time: f64) -> u32 {
    (new_time - old_time).abs() as u32
}

/// Formats how long ago something was published as a relative age (e.g. "1 Day",
/// "2 Weeks", "3 Months", "1 Year"), used for the podcast episode list. Takes "now"
/// as an explicit parameter rather than reading the clock internally, so this stays
/// a pure, testable function - both timestamps are milliseconds since the Unix
/// epoch, matching Audiobookshelf's own timestamp convention.
pub fn format_age(published_at_ms: i64, now_ms: i64) -> String {
    let days = (now_ms - published_at_ms).max(0) / (1000 * 60 * 60 * 24);

    fn pluralize(n: i64, unit: &str) -> String {
        if n == 1 { format!("1{unit}") } else { format!("{n}{unit}s") }
    }

    if days < 1 {
        "Today".to_string()
    } else if days < 7 {
        pluralize(days, "Day")
    } else if days < 30 {
        pluralize(days / 7, "Week")
    } else if days < 365 {
        pluralize(days / 30, "Month")
    } else {
        pluralize(days / 365, "Year")
    }
}

#[cfg(test)]
mod tests {
    use super::{format_age, progress_time_diff};

    #[test]
    fn forward_playback_gives_small_positive_diff() {
        assert_eq!(progress_time_diff(101.0, 100.0), 1);
    }

    #[test]
    fn backward_jump_does_not_panic_and_gives_absolute_diff() {
        // e.g. pressing `u` (jump back 10s) near the start of an episode
        assert_eq!(progress_time_diff(5.0, 15.0), 10);
    }

    const DAY_MS: i64 = 1000 * 60 * 60 * 24;

    #[test]
    fn same_day_shows_today() {
        assert_eq!(format_age(1000, 1000 + DAY_MS - 1), "Today");
    }

    #[test]
    fn single_day_is_singular() {
        assert_eq!(format_age(0, DAY_MS), "1Day");
    }

    #[test]
    fn a_few_days_shows_days() {
        assert_eq!(format_age(0, 2 * DAY_MS), "2Days");
    }

    #[test]
    fn single_week_is_singular() {
        assert_eq!(format_age(0, 10 * DAY_MS), "1Week");
    }

    #[test]
    fn a_few_weeks_shows_weeks() {
        assert_eq!(format_age(0, 20 * DAY_MS), "2Weeks");
    }

    #[test]
    fn a_few_months_shows_months() {
        assert_eq!(format_age(0, 60 * DAY_MS), "2Months");
    }

    #[test]
    fn over_a_year_shows_years() {
        assert_eq!(format_age(0, 400 * DAY_MS), "1Year");
    }
}

