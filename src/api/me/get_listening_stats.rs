use crate::utils::http_client::api_client;
use reqwest::header::AUTHORIZATION;
use color_eyre::eyre::{Result, Report};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Get listening stats for the current user - not scoped to any one library, since
/// Audiobookshelf tracks this per-user across all of them.
/// <https://api.audiobookshelf.org/#get-listening-stats>
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub total_time: Option<f64>,
    pub items: Option<HashMap<String, StatItem>>,
    pub days: Option<HashMap<String, f64>>,
    // Full English day names ("Monday", "Tuesday", ...), not numbers or abbreviations -
    // matches the server's own `date.format(d, 'dddd')`.
    pub day_of_week: Option<HashMap<String, f64>>,
    pub today: Option<f64>,
    pub recent_sessions: Option<Vec<Session>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatItem {
    pub id: Option<String>,
    pub time_listening: Option<f64>,
    pub media_metadata: Option<MediaMetadata>,
}

// Books and podcast episodes use different metadata field names server-side
// (authorName/narratorName vs. author) - both are folded into one struct here since
// `items` can hold either kind, rather than needing a mediaType-tagged enum for what's
// ultimately the same handful of optional strings.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub author_name: Option<String>,
    pub author: Option<String>,
    pub narrator_name: Option<String>,
    pub genres: Option<Vec<String>>,
}

impl MediaMetadata {
    // Multiple authors/narrators come back as one comma-joined string (see
    // Book.js's `narratorName: (this.narrators || []).join(', ')` server-side),
    // not an array - split back out here so callers aggregate per-person, not
    // per-combination.
    pub fn author_names(&self) -> Vec<String> {
        split_joined_names(self.author_name.as_deref().or(self.author.as_deref()))
    }

    pub fn narrator_names(&self) -> Vec<String> {
        split_joined_names(self.narrator_name.as_deref())
    }
}

fn split_joined_names(joined: Option<&str>) -> Vec<String> {
    joined
        .map(|s| s.split(',').map(|n| n.trim().to_string()).filter(|n| !n.is_empty()).collect())
        .unwrap_or_default()
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub display_title: Option<String>,
    pub display_author: Option<String>,
    pub time_listening: Option<f64>,
    pub date: Option<String>,
    pub updated_at: Option<i64>,
}

pub async fn get_listening_stats(token: &str, server_address: String) -> Result<Root> {
    let client = api_client();
    let url = format!("{server_address}/api/me/listening-stats");

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Report::new(std::io::Error::other(
                    "Failed to fetch data from the API",
        )));
    }

    let stats: Root = response.json().await?;

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_comma_joined_names_and_trims_whitespace() {
        let meta = MediaMetadata {
            author_name: Some("J.K. Rowling, Someone Else".to_string()),
            narrator_name: Some("Jim Dale".to_string()),
            ..Default::default()
        };
        assert_eq!(meta.author_names(), vec!["J.K. Rowling", "Someone Else"]);
        assert_eq!(meta.narrator_names(), vec!["Jim Dale"]);
    }

    #[test]
    fn falls_back_to_podcast_author_field_when_author_name_is_absent() {
        let meta = MediaMetadata {
            author: Some("NPR".to_string()),
            ..Default::default()
        };
        assert_eq!(meta.author_names(), vec!["NPR"]);
    }

    #[test]
    fn missing_names_produce_an_empty_list_not_a_panic() {
        let meta = MediaMetadata::default();
        assert!(meta.author_names().is_empty());
        assert!(meta.narrator_names().is_empty());
    }
}
