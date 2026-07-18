use reqwest::Client;
use serde_json::Value;
use reqwest::header::AUTHORIZATION;
use color_eyre::eyre::{Result, Report};
use serde::Deserialize;
use serde::Serialize;


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub id: String,
    pub user_id: String,
    pub library_item_id: String,
    pub episode_id: Value,
    pub media_item_id: String,
    pub media_item_type: String,
    pub duration: f64,
    pub progress: f64,
    pub current_time: f64,
    pub is_finished: bool,
    pub hide_from_continue_listening: bool,
    pub ebook_location: Value,
    pub ebook_progress: i64,
    pub last_update: i64,
    pub started_at: i64,
    pub finished_at: Value,
}

/// This endpoint retrieves your media progress that is associated with the given library item ID or podcast episode ID.
/// https://api.audiobookshelf.org/#get-a-media-progress

// get progress for a book
pub async fn get_book_progress(token: &str, book_id: &String, server_address: String) -> Result<Root> {
    let client = Client::new();
    let url = format!("{}/api/me/progress/{}", server_address, book_id);

    // Send GET request
    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?;

    // Check response status
    if !response.status().is_success() {
        return Err(Report::new(std::io::Error::other(
                    "Failed to fetch data from the API",
        )));
    }

    // Deserialize JSON response into Vec<Root>
    let book_progress: Root = response.json().await?;
    Ok(book_progress)
}

