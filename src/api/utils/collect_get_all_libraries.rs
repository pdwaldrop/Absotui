use crate::api::libraries::get_all_libraries::Root;


/// collect `media_type` (podcast or book)
pub async fn collect_media_types(library: &Root) -> Vec<String> {
    let mut media_types = Vec::new();

    for lib in &library.libraries {
        media_types.push(lib.media_type.clone());
    }

    media_types
}

/// `library_names`
pub async fn collect_library_names(library: &Root) -> Vec<String> {
    let mut library_names = Vec::new();

    for lib in &library.libraries {
        library_names.push(lib.name.clone());
    }

    library_names
}

/// collect `library_ids`
pub async fn collect_library_ids(library: &Root) -> Vec<String> {
    let mut library_ids = Vec::new();

    for lib in &library.libraries {
        library_ids.push(lib.id.clone());
    }

    library_ids
}


