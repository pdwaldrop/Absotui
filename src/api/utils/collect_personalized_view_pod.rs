use crate::api::libraries::get_library_perso_view_pod::Root;
use crate::utils::convert_seconds::convert_seconds;

/// collect id pod for continue listening
pub async fn collect_ids_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    let mut ids_pod_cnt_list = Vec::new();

    for root in roots {
        if let Some(entities) = &root.entities {
            for entity in entities {
                if let Some(recent_episode) = &entity.recent_episode {
                    if let Some(library_item_id) = recent_episode.library_item_id.clone() {
                        ids_pod_cnt_list.push(library_item_id);
                    } else {
                        ids_pod_cnt_list.push("N/A".to_string());
                    }
                }
            }
        }
    }

    ids_pod_cnt_list
}

/// Collect subtitles from recent episodes
pub async fn collect_subtitles_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    let mut subtitles_pod_cnt_list = Vec::new();

    for root in roots {
        if let Some(entities) = &root.entities {
            for entity in entities {
                if let Some(recent_episode) = &entity.recent_episode {
                    if let Some(subtitle) = &recent_episode.subtitle {
                        subtitles_pod_cnt_list.push(subtitle.clone());
                    } else {
                        subtitles_pod_cnt_list.push("N/A".to_string());
                    }

                }
            }
        }
    }

    subtitles_pod_cnt_list
}

/// Collect num episode 
pub async fn collect_nums_ep_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    let mut nums_ep_pod_cnt_list = Vec::new();

    for root in roots {
        if let Some(entities) = &root.entities {
            for entity in entities {
                if let Some(recent_episode) = &entity.recent_episode {
                    if let Some(episode) = &recent_episode.episode {
                        nums_ep_pod_cnt_list.push(episode.clone());
                    } else {
                        nums_ep_pod_cnt_list.push("N/A".to_string());
                    }
                }
            }
        }
    }

    nums_ep_pod_cnt_list
}

/// collect season
pub async fn collect_seasons_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    let mut seasons_pod_cnt_list = Vec::new();

    for root in roots {
        if let Some(entities) = &root.entities {
            for entity in entities {
                if let Some(recent_episode) = &entity.recent_episode {
                    if let Some(season) = &recent_episode.season {
                        seasons_pod_cnt_list.push(season.clone());
                    } else {
                        seasons_pod_cnt_list.push("N/A".to_string());
                    }

                }
            }
        }
    }

    seasons_pod_cnt_list
}

/// Collect authors
pub async fn collect_authors_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    let mut authors_pod_cnt_list = Vec::new();

    for root in roots {
        if let Some(entities) = &root.entities {
            for entity in entities {
                if let Some(_recent_episode) = &entity.recent_episode
                    && let Some(media) = &entity.media
                        && let Some(metadata) = &media.metadata {
                            if let Some(author) = &metadata.author {
                                authors_pod_cnt_list.push(author.clone());
                            } else {
                                authors_pod_cnt_list.push("N/A".to_string());
                            }

                        }
            }
        }
    }

    authors_pod_cnt_list
}

/// Collect description
pub async fn collect_descs_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    let mut descs_pod_cnt_list = Vec::new();

    for root in roots {
        if let Some(entities) = &root.entities {
            for entity in entities {
                if let Some(_recent_episode) = &entity.recent_episode
                    && let Some(media) = &entity.media
                        && let Some(metadata) = &media.metadata {
                            if let Some(desc) = &metadata.description {
                                descs_pod_cnt_list.push(desc.clone());
                            } else {
                                descs_pod_cnt_list.push("N/A".to_string());
                            }

                        }
            }
        }
    }

    descs_pod_cnt_list
}

/// Collect podcast title
pub async fn collect_titles_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    let mut titles_pod_cnt_list = Vec::new();

    for root in roots {
        if let Some(entities) = &root.entities {
            for entity in entities {
                if let Some(_recent_episode) = &entity.recent_episode
                    && let Some(media) = &entity.media
                        && let Some(metadata) = &media.metadata {
                            if let Some(title) = &metadata.title {
                                titles_pod_cnt_list.push(title.clone());
                            } else {
                                titles_pod_cnt_list.push("N/A".to_string());
                            }

                        }
            }
        }
    }

    titles_pod_cnt_list
}

pub async fn collect_durations_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    let mut durations = Vec::new();

    for root in roots {
        if let Some(entities) = &root.entities {
            for entity in entities {
                if let Some(recent_episode) = &entity.recent_episode
                    && let Some(audio_file) = &recent_episode.audio_file {
                        if let Some(duration) = audio_file.duration {
                            durations.push(duration);
                        } else {
                            durations.push(0.0);
                        }

                    }
            }
        }
    }

    
    convert_seconds(durations)
}

/// collect ids ep 
pub async fn collect_ids_ep_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    let mut ids_ep_pod_cnt_list = Vec::new();

    for root in roots {
        if let Some(entities) = &root.entities {
            for entity in entities {
                if let Some(recent_episode) = &entity.recent_episode {
                    if let Some(id) = recent_episode.id.clone() {
                        ids_ep_pod_cnt_list.push(id);
                    } else {
                        ids_ep_pod_cnt_list.push("N/A".to_string());
                    }

                }
            }
        }
    }

    ids_ep_pod_cnt_list
}

/// collect titles pod for continue listening
pub async fn collect_titles_cnt_list_pod(roots: &[Root]) -> Vec<String> {
    let mut titles_cnt_list = Vec::new();


    for root in roots {
        if let Some(entities) = &root.entities {
            for entity in entities {
                if let Some(recent_episode) = &entity.recent_episode {
                    if let Some(title) = recent_episode.title.clone() {
                        titles_cnt_list.push(title);
                    } else {
                        titles_cnt_list.push("N/A".to_string());
                    }

                }
            }
        }
    }

    titles_cnt_list
}
