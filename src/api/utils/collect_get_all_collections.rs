use crate::api::libraries::get_all_collections::Root;
use std::collections::HashMap;

/// collect collection names
pub async fn collect_collection_names(collections: &Root) -> Vec<String> {
    let mut names = Vec::new();

    if let Some(results) = &collections.results {
        for collection in results {
            if let Some(name) = &collection.name {
                names.push(name.clone());
            } else {
                names.push("N/A".to_string());
            }
        }
    }

    names
}

/// collect, per collection, the indices into `ids_library` of its books - resolved
/// once here rather than duplicating title/author/etc. for each book
pub async fn collect_collection_book_indices(collections: &Root, ids_library: &[String]) -> Vec<Vec<usize>> {
    let id_to_index: HashMap<&str, usize> = ids_library
        .iter()
        .enumerate()
        .map(|(index, id)| (id.as_str(), index))
        .collect();

    let mut collection_book_indices = Vec::new();

    if let Some(results) = &collections.results {
        for collection in results {
            let mut indices = Vec::new();
            if let Some(books) = &collection.books {
                for book in books {
                    if let Some(id) = &book.id
                        && let Some(&index) = id_to_index.get(id.as_str()) {
                            indices.push(index);
                        }
                }
            }
            collection_book_indices.push(indices);
        }
    }

    collection_book_indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::libraries::get_all_collections::{Collection, Book};

    #[tokio::test]
    async fn maps_collection_books_to_library_indices_and_skips_unmatched_ids() {
        let ids_library = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];

        let root = Root {
            results: Some(vec![
                Collection {
                    id: Some("col1".into()),
                    name: Some("Fantasy Favorites".into()),
                    books: Some(vec![
                        Book { id: Some("c".into()) },
                        Book { id: Some("a".into()) },
                        // Not present in ids_library - should be skipped, not panic.
                        Book { id: Some("missing-item".into()) },
                    ]),
                },
                Collection {
                    id: Some("col2".into()),
                    name: Some("Empty Collection".into()),
                    books: Some(vec![]),
                },
            ]),
        };

        assert_eq!(
            collect_collection_names(&root).await,
            vec!["Fantasy Favorites".to_string(), "Empty Collection".to_string()],
        );
        assert_eq!(
            collect_collection_book_indices(&root, &ids_library).await,
            vec![vec![2usize, 0usize], vec![]],
        );
    }
}
