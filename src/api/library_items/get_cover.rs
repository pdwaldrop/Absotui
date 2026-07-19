use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use color_eyre::eyre::Result;

/// Fetches a library item's (or podcast episode's parent item's) cover image as raw bytes.
/// <https://api.audiobookshelf.org/#get-item-cover>
pub async fn get_cover(token: &str, item_id: &str, server_address: &str) -> Result<Vec<u8>> {
    let client = Client::new();
    let url = format!("{server_address}/api/items/{item_id}/cover");

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?;

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}
