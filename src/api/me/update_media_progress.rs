use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::error::Error;

/// Create/Update Media Progress
/// This endpoint creates/updates your media progress for a library item or podcast episode.
/// <https://api.audiobookshelf.org/#create-update-media-progress>
// for a book
pub async fn update_media_progress_book(id_library_item: &str, token: Option<&String>, current_time: Option<u32>, duration: &str, server_adress: String) -> Result<(), Box<dyn Error>> {

    // Build client reqwest
    let client = reqwest::Client::new();

    // convert data before init progress (float)
    let duration_f32 = duration.parse::<f32>().unwrap();
    let current_time_f32: f32 = current_time.unwrap() as f32;

    // init  progress
    let progress = current_time_f32 / duration_f32 ;

    // json bosy
    let body = json!({
        "progress" : progress,
        "currentTime": current_time,
    });

    // Patch request
    let _response = client
        .patch(format!(
                "{server_adress}/api/me/progress/{id_library_item}"
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    // 
    //let status = response.status();
    //let response_text = response.text().await?;

    // println!("Statut: {}", status);
    // println!("Réponse: {}", response_text);

    Ok(())
}

// for a book (to mark as finished)
pub async fn update_media_progress2_book(id_library_item: &str, token: Option<&String>, current_time: Option<u32>, duration: &str, is_finished: bool, server_adress: String) -> Result<(), Box<dyn Error>> {

    // Build client reqwest
    let client = reqwest::Client::new();

    // convert data before init progress (float)
    let duration_f32 = duration.parse::<f32>().unwrap();
    let current_time_f32: f32 = current_time.unwrap() as f32;

    // init  progress
    let progress = current_time_f32 / duration_f32 ;

    // json bosy
    let body = json!({
        "progress" : progress,
        "isFinished" : is_finished,
        "currentTime": current_time,
    });

    // Patch request
    let _response = client
        .patch(format!(
                "{server_adress}/api/me/progress/{id_library_item}"
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .json(&body)
        .send()
        .await?;

    //
    //let status = response.status();
    //let response_text = response.text().await?;

    // println!("Statut: {}", status);
    // println!("Réponse: {}", response_text);

    Ok(())
}

// for a podcast : 
pub async fn update_media_progress_pod(id_library_item: &str , token: Option<&String>, current_time: Option<u32>, duration: &str, ep_id : &str, server_adress: String) -> Result<(), Box<dyn Error>> {

    // Build client reqwest
    let client = reqwest::Client::new();

    // convert data before init progress (float)
    let duration_f32 = duration.parse::<f32>().unwrap();
    let current_time_f32: f32 = current_time.unwrap() as f32;

    // init  progress
    let progress = current_time_f32 / duration_f32 ;

    // json bosy
    let body = json!({
        "progress" : progress,
        "currentTime": current_time,
    });

    // Patch request
    let _response = client
        .patch(format!(
                "{server_adress}/api/me/progress/{id_library_item}/{ep_id}"
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    // 
    //let status = response.status();
    //let response_text = response.text().await?;

    // println!("Statut: {}", status);
    // println!("Réponse: {}", response_text);

    Ok(())
}

// for a podcast (to mark as finished) : 
pub async fn update_media_progress2_pod(id_library_item: &str, token: Option<&String>, current_time: Option<u32>, duration: &str, is_finished: bool, ep_id: &str, server_adress: String) -> Result<(), Box<dyn Error>> {

    // Build client reqwest
    let client = reqwest::Client::new();

    // convert data before init progress (float)
    let duration_f32 = duration.parse::<f32>().unwrap();
    let current_time_f32: f32 = current_time.unwrap() as f32;

    // init  progress
    let progress = current_time_f32 / duration_f32 ;

    // json bosy
    let body = json!({
        "progress" : progress,
        "isFinished" : is_finished,
        "currentTime": current_time,
    });

    // Patch request
    let _response = client
        .patch(format!(
                "{server_adress}/api/me/progress/{id_library_item}/{ep_id}"
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .json(&body)
        .send()
        .await?;

    //
    //let status = response.status();
    //let response_text = response.text().await?;

    // println!("Statut: {}", status);
    // println!("Réponse: {}", response_text);

    Ok(())
}

