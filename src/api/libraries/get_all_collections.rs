use crate::utils::http_client::api_client;
use reqwest::header::AUTHORIZATION;
use color_eyre::eyre::{Result, Report};
use serde::Deserialize;
use serde::Serialize;

/// Get all collections in a library
/// <https://api.audiobookshelf.org/#get-all-collections-for-a-library>

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub results: Option<Vec<Collection>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: Option<String>,
    pub name: Option<String>,
    pub books: Option<Vec<Book>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: Option<String>,
}

pub async fn get_all_collections(token: &str, id_selected_lib: &String, server_address: String) -> Result<Root> {
    let client = api_client();
    let url = format!("{server_address}/api/libraries/{id_selected_lib}/collections");

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Report::new(std::io::Error::other(
                    "Failed to fetch data from the API",
        )));
    }

    let collections: Root = response.json().await?;

    Ok(collections)
}
