use reqwest::Client;
use serde_json::Value;
use reqwest::header::AUTHORIZATION;
use color_eyre::eyre::{Result, Report};
use serde::Deserialize;
use serde::Serialize;
use log::error;


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub id: String,
    pub user_id: String,
    pub library_item_id: String,
    pub episode_id: Value,
    pub media_item_id: String,
    pub media_item_type: String,
    #[serde(default)]
    pub duration: Option<f64>,
    pub progress: f64,
    pub current_time: f64,
    pub is_finished: bool,
    #[serde(default)]
    pub hide_from_continue_listening: Option<bool>,
    pub ebook_location: Value,
    #[serde(default)]
    pub ebook_progress: Option<f64>,
    #[serde(default)]
    pub last_update: Option<i64>,
    #[serde(default)]
    pub started_at: Option<i64>,
    pub finished_at: Value,
}

/// This endpoint retrieves your media progress that is associated with the given library item ID or podcast episode ID.
/// <https://api.audiobookshelf.org/#get-a-media-progress>
// get progress for a book
pub async fn get_book_progress(token: &str, book_id: &String, server_address: String) -> Result<Root> {
    let client = Client::new();
    let url = format!("{server_address}/api/me/progress/{book_id}");

    // Send GET request
    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?;

    // Check response status
    if !response.status().is_success() {
        return Err(Report::new(std::io::Error::other(
                    "Failed to fetch data from the API",
        )));
    }

    // Deserialize JSON response into Root - fetched as text first so a failure can log
    // the raw payload, since reqwest's own decode-error message doesn't show which field
    // or value actually failed to parse.
    let body_text = response.text().await?;
    match serde_json::from_str::<Root>(&body_text) {
        Ok(book_progress) => Ok(book_progress),
        Err(e) => {
            error!("[get_book_progress] failed to parse response for {book_id}: {e}\nraw body: {body_text}");
            Err(Report::new(e))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::Root;

    // Real payload from a book that also has ebook reading progress recorded
    // alongside its audiobook progress - `ebookProgress` comes back as a fraction
    // (f64), not an integer, which previously failed to deserialize.
    #[test]
    fn parses_progress_with_ebook_progress_fraction() {
        let raw = r#"{"id":"405dd912-56ef-4169-a7fd-a143db9ec386","userId":"4d80eafb-8c7e-4a3e-867d-be97b293c1c5","libraryItemId":"ee26c2cd-2317-4ec8-b60a-d298aa1eef40","episodeId":null,"mediaItemId":"1e37a1e8-b73a-472f-98d1-b0d291626f02","mediaItemType":"book","duration":37068.113333,"progress":0.6251214146614075,"currentTime":23172,"isFinished":false,"hideFromContinueListening":false,"ebookLocation":"24","ebookProgress":0.30666666666666664,"lastUpdate":1784412231758,"startedAt":1778465557297,"finishedAt":null}"#;
        let parsed: Root = serde_json::from_str(raw).expect("should parse now that ebook_progress is f64");
        assert_eq!(parsed.current_time, 23172.0);
        assert!((parsed.progress - 0.6251214146614075).abs() < f64::EPSILON);
    }
}
