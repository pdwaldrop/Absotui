use reqwest::Client;
use serde_json::Value;
use reqwest::header::AUTHORIZATION;
use color_eyre::eyre::{Result, Report};
use serde::Deserialize;
use serde::Serialize;


/// Get a Library Item, used for collect podact info (allow in particular to retrieve all podcast episode id)
/// This endpoint retrieves a library item, allow in particular to retrieve all podcast episode id.
/// https://api.audiobookshelf.org/#get-a-library-item

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub id: Option<String>,
    pub ino: Option<String>,
    pub old_library_item_id: Option<Value>,
    pub library_id: Option<String>,
    pub folder_id: Option<String>,
    pub path: Option<String>,
    pub rel_path: Option<String>,
    pub is_file: Option<bool>,
    pub mtime_ms: Option<i64>,
    pub ctime_ms: Option<i64>,
    pub birthtime_ms: Option<i64>,
    pub added_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub scan_version: Option<Value>,
    pub is_missing: Option<bool>,
    pub is_invalid: Option<bool>,
    pub media_type: Option<String>,
    pub media: Option<Media>,
    pub library_files: Option<Vec<LibraryFile>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: Option<String>,
    pub library_item_id: Option<String>,
    pub metadata: Option<Metadata>,
    pub cover_path: Option<String>,
    pub tags: Option<Vec<Value>>,
    pub episodes: Option<Vec<Episode>>,
    pub auto_download_episodes: Option<bool>,
    pub auto_download_schedule: Option<String>,
    pub last_episode_check: Option<i64>,
    pub max_episodes_to_keep: Option<i64>,
    pub max_new_episodes_to_download: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub genres: Option<Vec<String>>,
    pub feed_url: Option<String>,
    pub image_url: Option<String>,
    pub itunes_page_url: Option<String>,
    pub itunes_id: Option<Value>,
    pub itunes_artist_id: Option<Value>,
    pub explicit: Option<bool>,
    pub language: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub library_item_id: Option<String>,
    pub podcast_id: Option<String>,
    pub id: Option<String>,
    pub old_episode_id: Option<Value>,
    pub index: Option<Value>,
    pub season: Option<String>,
    pub episode: Option<String>,
    pub episode_type: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub enclosure: Option<Enclosure>,
    pub guid: Option<String>,
    pub pub_date: Option<String>,
    pub chapters: Option<Vec<Value>>,
    pub audio_file: Option<AudioFile>,
    pub published_at: Option<i64>,
    pub added_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enclosure {
    pub url: Option<String>,
    pub length: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioFile {
    pub path: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFile {
    pub file_name: Option<String>,
    pub file_path: Option<String>,
}



pub async fn get_pod_ep(token: &str, server_address: String, id: &str) -> Result<Root> {
    let client = Client::new();
    let url = format!("{}/api/items/{}", server_address, id);


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
    let item: Root = response.json().await?;

    Ok(item)
}


