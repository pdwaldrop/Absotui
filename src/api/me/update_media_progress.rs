use crate::utils::http_client::api_client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::error::Error;
use log::{info, error};

/// Create/Update Media Progress
/// This endpoint creates/updates your media progress for a library item or podcast episode.
/// <https://api.audiobookshelf.org/#create-update-media-progress>
pub async fn update_media_progress_book(id_library_item: &str, token: Option<&String>, current_time: Option<u32>, duration: &str, server_adress: String) -> Result<(), Box<dyn Error>> {
    let client = api_client();

    let duration_f32 = duration.parse::<f32>().unwrap();
    let current_time_f32: f32 = current_time.unwrap() as f32;
    let progress = current_time_f32 / duration_f32;

    let body = json!({
        "progress" : progress,
        "currentTime": current_time,
    });

    let _response = client
        .patch(format!(
                "{server_adress}/api/me/progress/{id_library_item}"
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    Ok(())
}

// for a book (to mark as finished)
pub async fn update_media_progress2_book(id_library_item: &str, token: Option<&String>, current_time: Option<u32>, duration: &str, is_finished: bool, server_adress: String) -> Result<(), Box<dyn Error>> {
    let client = api_client();

    let duration_f32 = duration.parse::<f32>().unwrap();
    let current_time_f32: f32 = current_time.unwrap() as f32;
    let progress = current_time_f32 / duration_f32;

    let body = json!({
        "progress" : progress,
        "isFinished" : is_finished,
        "currentTime": current_time,
    });

    let _response = client
        .patch(format!(
                "{server_adress}/api/me/progress/{id_library_item}"
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .json(&body)
        .send()
        .await?;

    Ok(())
}

pub async fn update_media_progress_pod(id_library_item: &str , token: Option<&String>, current_time: Option<u32>, duration: &str, ep_id : &str, server_adress: String) -> Result<(), Box<dyn Error>> {
    let client = api_client();

    let duration_f32 = duration.parse::<f32>().unwrap();
    let current_time_f32: f32 = current_time.unwrap() as f32;
    let progress = current_time_f32 / duration_f32;

    let body = json!({
        "progress" : progress,
        "currentTime": current_time,
    });

    let _response = client
        .patch(format!(
                "{server_adress}/api/me/progress/{id_library_item}/{ep_id}"
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    Ok(())
}

// for a podcast (to mark as finished)
pub async fn update_media_progress2_pod(id_library_item: &str, token: Option<&String>, current_time: Option<u32>, duration: &str, is_finished: bool, ep_id: &str, server_adress: String) -> Result<(), Box<dyn Error>> {
    let client = api_client();

    let duration_f32 = duration.parse::<f32>().unwrap();
    let current_time_f32: f32 = current_time.unwrap() as f32;
    let progress = current_time_f32 / duration_f32;

    let body = json!({
        "progress" : progress,
        "isFinished" : is_finished,
        "currentTime": current_time,
    });

    let response = client
        .patch(format!(
                "{server_adress}/api/me/progress/{id_library_item}/{ep_id}"
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        info!("[update_media_progress2_pod] marked {ep_id} isFinished={is_finished} - status {status}");
    } else {
        let response_text = response.text().await.unwrap_or_default();
        error!("[update_media_progress2_pod] failed to mark {ep_id} isFinished={is_finished} - status {status}, body: {response_text}");
    }

    Ok(())
}
