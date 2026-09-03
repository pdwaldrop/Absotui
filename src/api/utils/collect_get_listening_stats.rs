use crate::api::me::get_listening_stats::{MediaMetadata, Root, StatItem};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct StatsSummary {
    pub total_time: f64,
    pub today: f64,
    pub this_week: f64,
    pub this_month: f64,
    pub current_streak: u32,
    pub best_streak: u32,
    pub days_active: usize,
    /// Oldest to newest, always exactly 7 entries (0.0 for days with no listening).
    pub last_7_days: Vec<(NaiveDate, f64)>,
    /// Monday to Sunday, each `dayOfWeek`'s all-time total divided by the number of
    /// distinct calendar weeks any listening was recorded in - an approximate weekly
    /// average, not an ever-growing raw total.
    pub day_of_week_avg: [(Weekday, f64); 7],
    pub top_items: Vec<(String, f64)>,
    pub top_authors: Vec<(String, f64)>,
    pub top_narrators: Vec<(String, f64)>,
    pub top_genres: Vec<(String, f64)>,
}

const TOP_N: usize = 5;

impl Default for StatsSummary {
    fn default() -> Self {
        let order = [Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun];
        Self {
            total_time: 0.0,
            today: 0.0,
            this_week: 0.0,
            this_month: 0.0,
            current_streak: 0,
            best_streak: 0,
            days_active: 0,
            last_7_days: Vec::new(),
            day_of_week_avg: order.map(|wd| (wd, 0.0)),
            top_items: Vec::new(),
            top_authors: Vec::new(),
            top_narrators: Vec::new(),
            top_genres: Vec::new(),
        }
    }
}

pub async fn collect_stats_summary(stats: &Root, today: NaiveDate) -> StatsSummary {
    let days = parse_days(stats.days.as_ref());
    let items = stats.items.clone().unwrap_or_default();
    let day_of_week = stats.day_of_week.clone().unwrap_or_default();

    let mut active_dates: Vec<NaiveDate> = days.iter().filter(|&(_, &s)| s > 0.0).map(|(&d, _)| d).collect();
    active_dates.sort();

    StatsSummary {
        total_time: stats.total_time.unwrap_or(0.0),
        today: stats.today.unwrap_or(0.0),
        this_week: sum_in_range(&days, week_start(today), today),
        this_month: sum_in_range(&days, today.with_day(1).unwrap_or(today), today),
        current_streak: current_streak(&active_dates, today),
        best_streak: best_streak(&active_dates),
        days_active: active_dates.len(),
        last_7_days: last_7_days(&days, today),
        day_of_week_avg: weekly_average_by_day_of_week(&day_of_week, &active_dates),
        top_items: top_items(&items),
        top_authors: top_by_person(&items, MediaMetadata::author_names),
        top_narrators: top_by_person(&items, MediaMetadata::narrator_names),
        top_genres: top_genres(&items),
    }
}

// The server only ever emits an entry once real listening happened that day (see
// getUserListeningStatsHelpers server-side), but a hand-edited or otherwise malformed
// date key shouldn't be able to panic user-facing display code - skipped instead.
fn parse_days(days: Option<&HashMap<String, f64>>) -> HashMap<NaiveDate, f64> {
    days.into_iter()
        .flatten()
        .filter_map(|(date_str, &seconds)| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok().map(|d| (d, seconds)))
        .collect()
}

fn week_start(today: NaiveDate) -> NaiveDate {
    today - Duration::days(i64::from(today.weekday().num_days_from_monday()))
}

fn sum_in_range(days: &HashMap<NaiveDate, f64>, from: NaiveDate, to: NaiveDate) -> f64 {
    days.iter().filter(|&(&d, _)| d >= from && d <= to).map(|(_, &s)| s).sum()
}

/// Consecutive active days ending today or yesterday - today not having any listening
/// yet (it's still in progress) shouldn't zero out a streak that's still alive as of
/// yesterday.
fn current_streak(active_dates_sorted: &[NaiveDate], today: NaiveDate) -> u32 {
    let active: HashSet<NaiveDate> = active_dates_sorted.iter().copied().collect();
    let mut cursor = if active.contains(&today) {
        today
    } else if let Some(yesterday) = today.pred_opt().filter(|d| active.contains(d)) {
        yesterday
    } else {
        return 0;
    };

    let mut streak = 0u32;
    while active.contains(&cursor) {
        streak += 1;
        match cursor.pred_opt() {
            Some(d) => cursor = d,
            None => break,
        }
    }
    streak
}

fn best_streak(active_dates_sorted: &[NaiveDate]) -> u32 {
    let mut best = 0u32;
    let mut current = 0u32;
    let mut prev: Option<NaiveDate> = None;
    for &d in active_dates_sorted {
        current = if prev.and_then(|p| p.succ_opt()) == Some(d) { current + 1 } else { 1 };
        best = best.max(current);
        prev = Some(d);
    }
    best
}

fn last_7_days(days: &HashMap<NaiveDate, f64>, today: NaiveDate) -> Vec<(NaiveDate, f64)> {
    (0..7).rev().map(|i| {
        let d = today - Duration::days(i);
        (d, days.get(&d).copied().unwrap_or(0.0))
    }).collect()
}

fn weekday_name(wd: Weekday) -> &'static str {
    match wd {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

fn weekly_average_by_day_of_week(day_of_week: &HashMap<String, f64>, active_dates_sorted: &[NaiveDate]) -> [(Weekday, f64); 7] {
    let weeks_observed = active_dates_sorted.iter()
        .map(|d| (d.iso_week().year(), d.iso_week().week()))
        .collect::<HashSet<_>>()
        .len()
        .max(1) as f64;

    let order = [Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun];
    let mut result = [(Weekday::Mon, 0.0); 7];
    for (i, &wd) in order.iter().enumerate() {
        let total = day_of_week.get(weekday_name(wd)).copied().unwrap_or(0.0);
        result[i] = (wd, total / weeks_observed);
    }
    result
}

fn top_items(items: &HashMap<String, StatItem>) -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = items.values()
        .map(|item| {
            let title = item.media_metadata.as_ref()
                .and_then(|m| m.title.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            (title, item.time_listening.unwrap_or(0.0))
        })
        .collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1));
    v.truncate(TOP_N);
    v
}

fn top_by_person(items: &HashMap<String, StatItem>, extract: impl Fn(&MediaMetadata) -> Vec<String>) -> Vec<(String, f64)> {
    let mut totals: HashMap<String, f64> = HashMap::new();
    for item in items.values() {
        let Some(meta) = &item.media_metadata else { continue };
        let seconds = item.time_listening.unwrap_or(0.0);
        for name in extract(meta) {
            *totals.entry(name).or_insert(0.0) += seconds;
        }
    }
    let mut v: Vec<(String, f64)> = totals.into_iter().collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1));
    v.truncate(TOP_N);
    v
}

fn top_genres(items: &HashMap<String, StatItem>) -> Vec<(String, f64)> {
    let mut totals: HashMap<String, f64> = HashMap::new();
    for item in items.values() {
        let Some(meta) = &item.media_metadata else { continue };
        let seconds = item.time_listening.unwrap_or(0.0);
        for genre in meta.genres.clone().unwrap_or_default() {
            *totals.entry(genre).or_insert(0.0) += seconds;
        }
    }
    let mut v: Vec<(String, f64)> = totals.into_iter().collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1));
    v.truncate(TOP_N);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn current_streak_counts_back_from_today_when_today_is_active() {
        let dates = vec![d("2026-09-01"), d("2026-09-02"), d("2026-09-03")];
        assert_eq!(current_streak(&dates, d("2026-09-03")), 3);
    }

    #[test]
    fn current_streak_counts_back_from_yesterday_when_today_has_no_listening_yet() {
        let dates = vec![d("2026-09-01"), d("2026-09-02")];
        assert_eq!(current_streak(&dates, d("2026-09-03")), 2);
    }

    #[test]
    fn current_streak_is_zero_when_neither_today_nor_yesterday_is_active() {
        let dates = vec![d("2026-08-20")];
        assert_eq!(current_streak(&dates, d("2026-09-03")), 0);
    }

    #[test]
    fn best_streak_finds_the_longest_run_even_if_it_is_not_the_most_recent() {
        // A 3-day run (1st-3rd), a gap, then a 1-day run (10th).
        let dates = vec![d("2026-09-01"), d("2026-09-02"), d("2026-09-03"), d("2026-09-10")];
        assert_eq!(best_streak(&dates), 3);
    }

    #[test]
    fn best_streak_is_zero_for_no_active_days() {
        assert_eq!(best_streak(&[]), 0);
    }

    #[test]
    fn last_7_days_always_has_seven_entries_oldest_first_zero_filled() {
        let mut days = HashMap::new();
        days.insert(d("2026-09-03"), 120.0);
        let result = last_7_days(&days, d("2026-09-03"));
        assert_eq!(result.len(), 7);
        assert_eq!(result[0].0, d("2026-08-28"));
        assert_eq!(result[6].0, d("2026-09-03"));
        assert_eq!(result[6].1, 120.0);
        assert_eq!(result[0].1, 0.0);
    }

    #[test]
    fn top_by_person_splits_comma_joined_names_and_sums_across_items() {
        let mut items = HashMap::new();
        items.insert("a".to_string(), StatItem {
            id: Some("a".to_string()),
            time_listening: Some(100.0),
            media_metadata: Some(MediaMetadata {
                author_name: Some("J.K. Rowling".to_string()),
                ..Default::default()
            }),
        });
        items.insert("b".to_string(), StatItem {
            id: Some("b".to_string()),
            time_listening: Some(50.0),
            media_metadata: Some(MediaMetadata {
                author_name: Some("J.K. Rowling, Someone Else".to_string()),
                ..Default::default()
            }),
        });
        let result = top_by_person(&items, MediaMetadata::author_names);
        assert_eq!(result[0], ("J.K. Rowling".to_string(), 150.0));
        assert!(result.contains(&("Someone Else".to_string(), 50.0)));
    }

    #[test]
    fn top_items_truncates_to_five_and_sorts_descending() {
        let mut items = HashMap::new();
        for i in 0..8 {
            items.insert(format!("id{i}"), StatItem {
                id: Some(format!("id{i}")),
                time_listening: Some(f64::from(i)),
                media_metadata: Some(MediaMetadata { title: Some(format!("Book {i}")), ..Default::default() }),
            });
        }
        let result = top_items(&items);
        assert_eq!(result.len(), 5);
        assert_eq!(result[0].0, "Book 7");
        assert_eq!(result[4].0, "Book 3");
    }
}
