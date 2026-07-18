use reqwest::Client; 
use color_eyre::eyre::Result; 
use reqwest::header::AUTHORIZATION;
use serde_json::Value;
use serde_json::json;
use crate::player::vlc::fetch_vlc_data::get_vlc_version;


const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Play a Library Item or Podcast Episode
/// This endpoint starts a playback session for a library item or podcast episode.
/// https://api.audiobookshelf.org/#play-a-library-item-or-podcast-episode

// play book 
pub async fn post_start_playback_session_book(token: Option<&String>, id_library_item: &str, server_address: String) -> Result<Vec<String>, reqwest::Error> {
    let mut vlc_version = String::new();
    match get_vlc_version().await {
        Ok(version) => {vlc_version = version;}
        Err(e) => {
            log::error!("[get_vlc_version] {}",e);
        }
    }
    let client = Client::new();

    let params = json!({
        "forceDirectPlay": true, // avoid latency load, allow view chapter, cover etc.(the .m3u8 stream the original format, ex: .m4b) when playing with vlc
        "mediaPlayer": format!("VLC v{}", vlc_version),
        "deviceInfo": {  
            "clientName": "Absotui",
            "clientVersion": format!("v{}", VERSION),
            // to have OS displayed in user activity pannel (audiobookshelf/config/users/)
            "manufacturer": format!("{}", std::env::consts::OS),
            "model": format!("{}", std::env::consts::ARCH),
        }});

    let response = client
        .post(format!(
                "{}/api/items/{}/play", 
                server_address,
                id_library_item
        ))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .json(&params)
        .send()
        .await?;

    // Retrieve JSON response
    let v: Value = response.json().await?;

    // Retrieve data
    let current_time = v["currentTime"]
        .as_f64()
        .unwrap_or(0.0);
    let content_url = v["audioTracks"][0]["contentUrl"]
        .as_str()
        .unwrap_or("");
    let duration = v["audioTracks"][0]["duration"]
        .as_f64()
        .unwrap_or(0.0);
    let duration: u32 = duration as u32;
    let id_session = v["id"]
        .as_str()
        .unwrap_or("");
    let title = v["mediaMetadata"]["title"]
        .as_str()
        .unwrap_or("N/A");
    let subtitle = v["mediaMetadata"]["title"]
        .as_str()
        .unwrap_or("N/A");
    let author = v["displayAuthor"]
        .as_str()
        .unwrap_or("N/A");

    let info_item = vec![
        current_time.to_string(), 
        content_url.to_string(), 
        duration.to_string(), 
        id_session.to_string(), 
        title.to_string(), 
        subtitle.to_string(), 
        author.to_string()
    ];

    Ok(info_item)
}
// play podcast episode
pub async fn post_start_playback_session_pod(token: Option<&String>, id_library_item: &str, pod_ep_id: &str, server_address: String) -> Result<Vec<String>, reqwest::Error> {
    let mut vlc_version = String::new();
    match get_vlc_version().await {
        Ok(version) => {vlc_version = version;}
        Err(_e) => {
            //eprintln!("{}", e),
        }
    }
    let client = Client::new();

    let params = json!({
        "forceDirectPlay": true, // avoid latency load, allow view chapter, cover etc.(the .m3u8 stream the original format, ex: .m4b) when playing with vlc
        "mediaPlayer": format!("VLC v{}", vlc_version),
        "deviceInfo": {  
            "clientName": "Absotui",
            "clientVersion": format!("v{}", VERSION),
            // to have OS displayed in user activity pannel (audiobookshelf/config/users/)
            "manufacturer": format!("{}", std::env::consts::OS),
            "model": format!("{}", std::env::consts::ARCH),
        }});

    let response = client
        .post(format!(
                "{}/api/items/{}/play/{}", 
                server_address,
                id_library_item, 
                pod_ep_id,
        ))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .json(&params)
        .send()
        .await?;

    // Retrieve JSON response
    let v: Value = response.json().await?;

    // Retrieve data
    let current_time = v["currentTime"]
        .as_f64()
        .unwrap_or(0.0);
    let content_url = v["audioTracks"][0]["contentUrl"]
        .as_str()
        .unwrap_or("");
    let duration = v["audioTracks"][0]["duration"]
        .as_f64()
        .unwrap_or(0.0);
    let duration: u32 = duration as u32;
    let id_session = v["id"]
        .as_str()
        .unwrap_or("");
    let title = v["mediaMetadata"]["title"]
        .as_str()
        .unwrap_or("N/A");
    let subtitle = v["displayTitle"]
        .as_str()
        .unwrap_or("N/A");
    let author = v["displayAuthor"]
        .as_str()
        .unwrap_or("N/A");

    let info_item = vec![
        current_time.to_string(), 
        content_url.to_string(), 
        duration.to_string(), 
        id_session.to_string(), 
        title.to_string(), 
        subtitle.to_string(), 
        author.to_string()
    ];

    Ok(info_item)
}
