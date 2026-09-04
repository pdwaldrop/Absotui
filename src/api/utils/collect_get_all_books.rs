use crate::api::libraries::get_all_books::{Root, Metadata};

/// collect titles - always one entry per `results` item (an item with missing
/// media/metadata still gets a placeholder), same as `collect_ids_library` -
/// callers index this array in lockstep with several others built the same way
/// (see CLAUDE.md's parallel-arrays warning), so skipping an item here instead
/// of the sibling arrays would silently misalign every index after it.
pub async fn collect_titles_library(library: &Root) -> Vec<String> {
    let mut titles_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            let title = item.media.as_ref()
                .and_then(|media| media.metadata.as_ref())
                .and_then(|metadata| metadata.title.clone());
            titles_library.push(title.unwrap_or_else(|| "N/A".to_string()));
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

/// collect author name for book - always one entry per `results` item, see
/// `collect_titles_library`'s doc comment.
pub async fn collect_auth_names_library(library: &Root) -> Vec<String> {
    let mut auth_names_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            let author_name = item.media.as_ref()
                .and_then(|media| media.metadata.as_ref())
                .and_then(|metadata| metadata.author_name.clone());
            auth_names_library.push(author_name.unwrap_or_else(|| "N/A".to_string()));
        }
    }

    auth_names_library
}

/// collect author name for podcast - always one entry per `results` item, see
/// `collect_titles_library`'s doc comment.
pub async fn collect_auth_names_library_pod(library: &Root) -> Vec<String> {
    let mut auth_names_library_pod = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            let author = item.media.as_ref()
                .and_then(|media| media.metadata.as_ref())
                .and_then(|metadata| metadata.author.clone());
            auth_names_library_pod.push(author.unwrap_or_else(|| "N/A".to_string()));
        }
    }

    auth_names_library_pod
}
/// collect published year - always one entry per `results` item, see
/// `collect_titles_library`'s doc comment.
pub async fn collect_published_year_library(library: &Root) -> Vec<String> {
    let mut published_year_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            let pub_year = item.media.as_ref()
                .and_then(|media| media.metadata.as_ref())
                .and_then(|metadata| metadata.published_year.clone());
            published_year_library.push(pub_year.unwrap_or_else(|| "N/A".to_string()));
        }
    }

    published_year_library
}

/// collect description - always one entry per `results` item, see
/// `collect_titles_library`'s doc comment.
pub async fn collect_desc_library(library: &Root) -> Vec<String> {
    let mut desc_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            let desc = item.media.as_ref()
                .and_then(|media| media.metadata.as_ref())
                .and_then(|metadata| metadata.description.clone());
            desc_library.push(desc.unwrap_or_else(|| "No description available".to_string()));
        }
    }

    desc_library
}

/// Resolves a book's primary series (name, sequence) - preferring the structured
/// `series` array Audiobookshelf's own API docs describe, falling back to parsing
/// the flat, packed `seriesName` string this server actually sends today (e.g. "The
/// Wheel of Time #3", or "Harry Potter #7, Wizarding World Collection #7" for a book
/// in more than one series - only the first, comma-separated, entry is used).
fn resolve_series(metadata: &Metadata) -> (Option<String>, Option<f64>) {
    if let Some(series) = metadata.series.as_ref().and_then(|s| s.first()) {
        return (series.name.clone(), series.sequence.as_ref().and_then(|s| s.parse().ok()));
    }

    let Some(raw) = &metadata.series_name else { return (None, None) };
    let first_entry = raw.split(',').next().unwrap_or(raw).trim();
    if first_entry.is_empty() {
        return (None, None);
    }
    match first_entry.rsplit_once(" #") {
        Some((name, sequence)) => (Some(name.trim().to_string()), sequence.trim().parse().ok()),
        None => (Some(first_entry.to_string()), None),
    }
}

/// collect each book's primary series name and sequence number (sequence parsed to
/// `f64` so a group sorts numerically - "2.5" between "2" and "3" - rather than
/// lexically) in one pass, since both come from the same `resolve_series` call per
/// book - collecting them separately would re-parse the same packed `seriesName`
/// string for every book twice.
pub async fn collect_series_library(library: &Root) -> (Vec<Option<String>>, Vec<Option<f64>>) {
    let mut series_name_library = Vec::new();
    let mut series_sequence_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            let (name, sequence) = item.media.as_ref()
                .and_then(|media| media.metadata.as_ref())
                .map(resolve_series)
                .unwrap_or((None, None));
            series_name_library.push(name);
            series_sequence_library.push(sequence);
        }
    }

    (series_name_library, series_sequence_library)
}

/// collect duration - always one entry per `results` item, see
/// `collect_titles_library`'s doc comment.
pub async fn collect_duration_library(library: &Root) -> Vec<f64> {
    let mut duration = vec![];

    if let Some(results) = &library.results {
        for item in results {
            let dur = item.media.as_ref().and_then(|media| media.duration);
            duration.push(dur.unwrap_or(0.0));
        }
    }

    duration
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::libraries::get_all_books::{LibraryItem, Media, Series};

    fn book_with_series(series: Option<Vec<Series>>) -> LibraryItem {
        LibraryItem {
            media: Some(Media { metadata: Some(Metadata { series, ..Default::default() }), ..Default::default() }),
            ..Default::default()
        }
    }

    fn book_with_series_name(series_name: Option<&str>) -> LibraryItem {
        LibraryItem {
            media: Some(Media {
                metadata: Some(Metadata { series_name: series_name.map(str::to_string), ..Default::default() }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn collects_first_series_entry_only_and_parses_sequence_numerically() {
        let root = Root {
            results: Some(vec![
                // Belongs to two series - only the first should be used.
                book_with_series(Some(vec![
                    Series { id: None, name: Some("Main Series".to_string()), sequence: Some("2.5".to_string()) },
                    Series { id: None, name: Some("Other Series".to_string()), sequence: Some("1".to_string()) },
                ])),
                // No series at all.
                book_with_series(None),
                // Series present but an unparsable sequence.
                book_with_series(Some(vec![
                    Series { id: None, name: Some("Odd Series".to_string()), sequence: Some("not-a-number".to_string()) },
                ])),
            ]),
            ..Default::default()
        };

        let (names, sequences) = collect_series_library(&root).await;
        assert_eq!(
            names,
            vec![Some("Main Series".to_string()), None, Some("Odd Series".to_string())],
        );
        assert_eq!(sequences, vec![Some(2.5), None, None]);
    }

    #[tokio::test]
    async fn falls_back_to_parsing_the_packed_series_name_string() {
        let root = Root {
            results: Some(vec![
                book_with_series_name(Some("The Wheel of Time #3")),
                // Two series packed into one comma-separated string - only the
                // first should be used, matching the array-based case above.
                book_with_series_name(Some("Harry Potter #7, Wizarding World Collection #7")),
                // A name with no "#sequence" suffix at all.
                book_with_series_name(Some("Standalone Series Name")),
                book_with_series_name(None),
            ]),
            ..Default::default()
        };

        let (names, sequences) = collect_series_library(&root).await;
        assert_eq!(
            names,
            vec![
                Some("The Wheel of Time".to_string()),
                Some("Harry Potter".to_string()),
                Some("Standalone Series Name".to_string()),
                None,
            ],
        );
        assert_eq!(sequences, vec![Some(3.0), Some(7.0), None, None]);
    }

    #[tokio::test]
    async fn structured_series_array_takes_priority_over_the_packed_string() {
        let root = Root {
            results: Some(vec![LibraryItem {
                media: Some(Media {
                    metadata: Some(Metadata {
                        series: Some(vec![Series { id: None, name: Some("Array Series".to_string()), sequence: Some("1".to_string()) }]),
                        series_name: Some("String Series #9".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let (names, sequences) = collect_series_library(&root).await;
        assert_eq!(names, vec![Some("Array Series".to_string())]);
        assert_eq!(sequences, vec![Some(1.0)]);
    }
}
