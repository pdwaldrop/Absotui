use reqwest::Client;
use serde_json::Value;
use reqwest::header::AUTHORIZATION;
use color_eyre::eyre::{Result, Report};
use serde::Deserialize;
use serde::Serialize;


/// Get All Libraries (can be a podcast or book library (shelf))
/// <https://api.audiobookshelf.org/#get-all-libraries>


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub libraries: Vec<Library>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub id: String,
    pub name: String,
    pub folders: Vec<Folder>,
    pub display_order: i64,
    pub icon: String,
    pub media_type: String,
    pub provider: String,
    pub settings: Settings,
    pub last_scan: Option<i64>,
    pub last_scan_version: Option<String>,
    pub created_at: i64,
    pub last_update: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub full_path: String,
    pub library_id: String,
    pub added_at: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub cover_aspect_ratio: i64,
    pub disable_watcher: bool,
    pub auto_scan_cron_expression: Value,
    pub skip_matching_media_with_asin: Option<bool>,
    pub skip_matching_media_with_isbn: Option<bool>,
    pub audiobooks_only: Option<bool>,
    pub epubs_allow_scripted_content: Option<bool>,
    pub hide_single_book_series: Option<bool>,
    pub only_show_later_books_in_continue_series: Option<bool>,
    pub metadata_precedence: Option<Vec<String>>,
    #[serde(default)]
    pub mark_as_finished_percent_complete: Value,
    #[serde(default)]
    pub mark_as_finished_time_remaining: i64,
    pub podcast_search_region: Option<String>,
}

// get all libraries (shelf). A library can be a Podcast or a Book type
pub async fn get_all_libraries(token: &str, server_address: String) -> Result<Root> {
    let client = Client::new();
    let url = format!("{server_address}/api/libraries");

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

    // Deserialize JSON response into Vec<Root>
    let libraries: Root = response.json().await?;

    Ok(libraries)
}
