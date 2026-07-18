use reqwest::Client; 
use color_eyre::eyre::Result; 
use reqwest::header::AUTHORIZATION;
use serde_json::json;

/// This endpoint syncs the position of an open listening session from the client to the server and returns the session.
/// <https://api.audiobookshelf.org/#sync-an-open-session>
// sync a session
pub async fn sync_session(token: Option<&String>, session_id: &str, current_time: Option<u32>, time_listened: u32, server_address: String) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let params = json!({
        "currentTime": format!("{}", current_time.unwrap_or(0)), 
        "timeListened": format!("{}", time_listened),
    });

    let _response = client
        .post(format!(
                "{server_address}/api/session/{session_id}/sync"
        ))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .json(&params)
        .send()
        .await?;

    Ok(())
}
