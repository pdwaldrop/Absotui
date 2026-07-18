use crate::api::libraries::get_all_books::Root;

/// collect titles
pub async fn collect_titles_library(library: &Root) -> Vec<String> {
    let mut titles_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media
                && let Some(metadata) = &media.metadata {
                    if let Some(title) = &metadata.title {
                        titles_library.push(title.clone());
                    } else {
                        titles_library.push("N/A".to_string());
                    }
                }
        }
    }

    titles_library
}

/// collect ID of library items 
pub async fn collect_ids_library(library: &Root) -> Vec<String> {
    let mut ids_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(id) = &item.id {
                ids_library.push(id.clone());
            } else {
                ids_library.push("N/A".to_string());
            }

        }
    }

    ids_library
}

/// collect author name for book
pub async fn collect_auth_names_library(library: &Root) -> Vec<String> {
    let mut auth_names_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media
                && let Some(metadata) = &media.metadata {
                    if let Some(author_name) = &metadata.author_name {
                        auth_names_library.push(author_name.clone());
                    } else {
                        auth_names_library.push("N/A".to_string());
                    }

                }
        }
    }

    auth_names_library
}

/// collect author name for podcast
pub async fn collect_auth_names_library_pod(library: &Root) -> Vec<String> {
    let mut auth_names_library_pod = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media
                && let Some(metadata) = &media.metadata {
                    if let Some(author) = &metadata.author {
                        auth_names_library_pod.push(author.clone());
                    } else {
                        auth_names_library_pod.push("N/A".to_string());
                    }

                }
        }
    }

    auth_names_library_pod
}
/// collect published year
pub async fn collect_published_year_library(library: &Root) -> Vec<String> {
    let mut published_year_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media
                && let Some(metadata) = &media.metadata {
                    if let Some(pub_year) = &metadata.published_year {
                        published_year_library.push(pub_year.clone());
                    } else {
                        published_year_library.push("N/A".to_string());
                    }

                }
        }
    }

    published_year_library
}

/// collect description
pub async fn collect_desc_library(library: &Root) -> Vec<String> {
    let mut desc_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media
                && let Some(metadata) = &media.metadata {
                    if let Some(desc) = &metadata.description {
                        desc_library.push(desc.clone());
                    } else {
                        desc_library.push("No description available".to_string());
                    }
                }
        }
    }

    desc_library
}

/// collect duration
pub async fn collect_duration_library(library: &Root) -> Vec<f64> {
    let mut duration = vec![];

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media {
                if let Some(dur) = &media.duration {
                    duration.push(*dur);
                } else {
                    duration.push(0.0);
                }

            }
        }
    }

    duration 
}
