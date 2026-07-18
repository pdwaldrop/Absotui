use reqwest::Client; 
use color_eyre::eyre::Result; 
use reqwest::header::AUTHORIZATION;

// This endpoint closes an open listening session. Optionally provide sync data to update the session before closing it.
// https://api.audiobookshelf.org/#close-an-open-session

pub async fn close_session_without_send_prg_data(token: Option<&String>, session_id: &str, server_address: String) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let _response = client
        .post(format!(
                "{server_address}/api/session/{session_id}/close"
        ))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", token.unwrap()))
        .send()
        .await?;

    Ok(())
}
