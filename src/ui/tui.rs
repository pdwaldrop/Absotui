use crate::App;
use crate::app::{AppView, HomeRow};
use crate::api::libraries::get_library_perso_view_pod::Chapter;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem , ListState,  Paragraph, StatefulWidget,
        Widget, Wrap
    },
};
use crate::utils::convert_seconds::{convert_seconds, convert_seconds_for_prg, format_age};
use crate::db::crud::{get_listening_session, get_is_podcast_autoplay};
use crate::player::integrated::player_info::{format_time, find_current_chapter};
use crate::config::load_config;


// const version
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// init widget for selected `AppView` 
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.view_state {
            AppView::Home => self.render_home(area, buf),
            AppView::Library => self.render_library(area, buf),
            AppView::SearchBook => self.render_search_book(area, buf),
            AppView::PodcastEpisode => self.render_pod_ep(area, buf),
            AppView::Settings => self.render_settings(area, buf),
            AppView::SettingsAccount => self.render_settings_account(area, buf),
            AppView::SettingsLibrary => self.render_settings_library(area, buf),
            AppView::SettingsAbout => {},
            AppView::SettingsUpdateUninstall => {},
            AppView::SettingsAutoplay => self.render_settings_autoplay(area, buf),
        }
    }
}


/// Rendering logic
impl App {
    /// `AppView::Home` rendering
    fn render_home(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);

        let items_number = self._titles_cnt_list.len();
        let render_list_title = if self.is_podcast {
            format!("New & Unfinished [{items_number} items]")
        } else {
            format!("Continue Listening [{items_number} items]")
        };

        let text_render_footer = if self.is_podcast {
            "j/↓, k/↑: move, l/→: play, Tab: library, R: refresh, S: Settings, Q/Esc: quit\n B: toggle player ctrl, D: sort by age, '/': search, Scroll desc: J(↓) K(↑) H(⇡), g/G: top/bot"
        } else {
            "j/↓, k/↑: move, l/→: play, c: chapters, Tab: library, R: refresh, S: Settings, Q/Esc: quit\n B: toggle player ctrl, '/': search, Scroll desc: J(↓) K(↑) H(⇡), g/G: top/bot"
        };

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);

        // Pin the actively-playing item to the top. Runs on every render (not just on
        // load/refresh) so it reacts as soon as playback starts. Books match by id_item;
        // podcasts must match by episode ID (id_pod) since id_item there is the parent
        // podcast's ID, which multiple episodes in this list could share.
        if let Ok(Some(active_session)) = get_listening_session() {
            if !self.is_podcast
                && let Some(pos) = self._ids_cnt_list.iter().position(|id| id == &active_session.id_item)
                && pos != 0 {
                    let selected_id = self.list_state_cnt_list.selected()
                        .and_then(|i| self._ids_cnt_list.get(i))
                        .cloned();

                    let mut order: Vec<usize> = (0..self._ids_cnt_list.len()).collect();
                    order.remove(pos);
                    order.insert(0, pos);

                    self._titles_cnt_list = order.iter().map(|&i| self._titles_cnt_list[i].clone()).collect();
                    self.auth_names_cnt_list = order.iter().map(|&i| self.auth_names_cnt_list[i].clone()).collect();
                    self.pub_year_cnt_list = order.iter().map(|&i| self.pub_year_cnt_list[i].clone()).collect();
                    self.duration_cnt_list = order.iter().map(|&i| self.duration_cnt_list[i]).collect();
                    self.desc_cnt_list = order.iter().map(|&i| self.desc_cnt_list[i].clone()).collect();
                    self._ids_cnt_list = order.iter().map(|&i| self._ids_cnt_list[i].clone()).collect();
                    self.book_progress_cnt_list = order.iter().map(|&i| self.book_progress_cnt_list[i].clone()).collect();
                    self.book_progress_cnt_list_cur_time = order.iter().map(|&i| self.book_progress_cnt_list_cur_time[i].clone()).collect();

                    if let Some(id) = selected_id
                        && let Some(new_pos) = self._ids_cnt_list.iter().position(|i| *i == id) {
                            self.list_state_cnt_list.select(Some(new_pos));
                    }
            }

            if self.is_podcast
                && let Some(pos) = self.ids_ep_cnt_list.iter().position(|id| id == &active_session.id_pod)
                && pos != 0 {
                    let selected_ep_id = self.list_state_cnt_list.selected()
                        .and_then(|i| self.ids_ep_cnt_list.get(i))
                        .cloned();

                    let mut order: Vec<usize> = (0..self.ids_ep_cnt_list.len()).collect();
                    order.remove(pos);
                    order.insert(0, pos);
                    self.reorder_podcast_lists(&order);

                    if let Some(id) = selected_ep_id
                        && let Some(new_pos) = self.ids_ep_cnt_list.iter().position(|i| *i == id) {
                            self.list_state_cnt_list.select(Some(new_pos));
                    }
            }
        }

        // Which item (if any) matches the actual active listening session - distinct from
        // wherever the cursor/highlight currently happens to be sitting in the list. Books
        // match by id_item; podcasts must match by episode ID (id_pod), same reasoning as
        // the reorder above.
        let active_session = get_listening_session().ok().flatten();
        let now_playing_id: Option<String> = active_session.as_ref().map(|s| if self.is_podcast { s.id_pod.clone() } else { s.id_item.clone() });

        // Flattened book/chapter rows - plain Book rows 1:1 with _ids_cnt_list unless the
        // chapter list is expanded, in which case it also carries the chapter sub-rows to
        // render beneath the currently-playing book. Kept in sync with input handling
        // (app.rs) since both go through this same method.
        let home_rows = self.build_home_rows();
        let current_chapter_id: Option<i64> = if self.is_chapter_list_expanded {
            active_session.as_ref().and_then(|s| {
                let chapters: Vec<Chapter> = serde_json::from_str(&s.chapters).unwrap_or_default();
                find_current_chapter(&chapters, s.current_time as f64).and_then(|c| c.id)
            })
        } else {
            None
        };

        let progress_info: Option<Vec<(String, f32, bool)>> = if self.is_podcast {
            // Progress percent isn't shown here - it isn't as meaningful for a list
            // already filtered to "new or unfinished" episodes. Instead the time slot
            // shows the episode's age (e.g. "1Day", "2Weeks"), with percent forced to
            // 0.0 so it renders as plain text with no underline fill.
            //
            // Left-aligned within a fixed-width field (trailing spaces, not leading) so
            // the leading character (the digit, or the "T" of "Today") lands in the same
            // column on every row - right-padding instead of left-padding, since the
            // labels vary in length and right-aligning only lines up their trailing
            // edge. Width is wide enough for the longest realistic label ("12Months").
            const AGE_LABEL_WIDTH: usize = 8;
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            Some(self._titles_cnt_list.iter().enumerate().map(|(i, _)| {
                let is_now_playing = self.ids_ep_cnt_list.get(i).is_some_and(|id| Some(id) == now_playing_id.as_ref());
                let age = self.podcast_published_at_cnt_list.get(i)
                    .map(|&published_at| format_age(published_at, now_ms))
                    .unwrap_or_default();
                (format!("{age:<AGE_LABEL_WIDTH$}"), 0.0, is_now_playing)
            }).collect())
        } else {
            Some(home_rows.iter().map(|row| match row {
                HomeRow::Book(i) => {
                    let i = *i;
                    let is_now_playing = self._ids_cnt_list.get(i).is_some_and(|id| Some(id) == now_playing_id.as_ref());
                    let duration = self.duration_cnt_list.get(i).copied().unwrap_or(0.0) as f32;

                    // For the actively-playing book, use the live position from the local
                    // listening_session (updated every second while VLC plays) instead of the
                    // snapshot fetched from the server when the list last loaded - keeps this
                    // one row's progress current without any extra network calls.
                    let current_time = if is_now_playing {
                        active_session.as_ref().map(|s| s.current_time as f32).unwrap_or(0.0)
                    } else {
                        self.book_progress_cnt_list_cur_time.get(i).and_then(|v| v.first()).copied().unwrap_or(0.0) as f32
                    };
                    let percent = if is_now_playing && duration > 0.0 {
                        (current_time / duration) * 100.0
                    } else {
                        self.book_progress_cnt_list.get(i)
                            .and_then(|v| v.first())
                            .and_then(|s| s.trim().parse::<f32>().ok())
                            .unwrap_or(0.0)
                    };
                    // Gate on the raw current_time, not the rounded percent string - a book
                    // with small-but-real progress (e.g. 0.3% into an 11-hour audiobook) would
                    // round to "0" and get misreported as never started.
                    let text = if current_time > 0.0 {
                        format!("{} / {} ({}%)", format_time(current_time as u32), format_time(duration as u32), percent.round() as u32)
                    } else {
                        "Not started".to_string()
                    };
                    (text, percent, is_now_playing)
                }
                // Chapter rows render as plain indented rows (no time text/underline, no
                // now-playing marker box) - which chapter is current is shown inline in
                // the title itself instead, see display_titles below.
                HomeRow::Chapter { .. } => (String::new(), 0.0, false),
            }).collect())
        };
        // Podcasts: show "Episode Title | Podcast Title" in the list row, not just the
        // episode title alone - _titles_cnt_list is episode titles, titles_pod_cnt_list
        // is the parent podcast's own title.
        let display_titles: Vec<String> = if self.is_podcast {
            self._titles_cnt_list.iter().enumerate().map(|(i, ep_title)| {
                match self.titles_pod_cnt_list.get(i) {
                    Some(pod_title) => format!("{ep_title} | {pod_title}"),
                    None => ep_title.clone(),
                }
            }).collect()
        } else {
            home_rows.iter().map(|row| match row {
                HomeRow::Book(i) => self._titles_cnt_list.get(*i).cloned().unwrap_or_default(),
                HomeRow::Chapter { chapter, .. } => {
                    let num = chapter.id.unwrap_or(0) + 1;
                    let title = chapter.title.clone().unwrap_or_default();
                    let is_current_chapter = chapter.id.is_some() && chapter.id == current_chapter_id;
                    let marker = if is_current_chapter { "●" } else { " " };
                    format!("    {marker} Chapter {num}. {title}")
                }
            }).collect()
        };
        self.render_list(list_area, buf, &render_list_title, &display_titles, &mut self.list_state_cnt_list.clone(), progress_info.as_deref());
        if !&self._titles_cnt_list.is_empty() {
            self.render_info_home(item_area1, buf, &self.list_state_cnt_list.clone());
            self.render_desc_home(item_area2, buf, &self.list_state_cnt_list.clone());
        }
    }

    /// `AppView::Library` rendering
    fn render_library(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);

        let items_number = self.titles_library.len();
        let render_list_title = format!("Library [{items_number} items]");

        let mut _text_render_footer = "";
        if self.is_podcast {
        _text_render_footer = "j/↓, k/↑: move, l/→: episodes, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n B: toggle player ctrl, '/': search, Scroll desc: J(↓) K(↑) H(⇡), g/G: top/bot";       
        } else {
        _text_render_footer = "j/↓, k/↑: move, l/→: play, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n B: toggle player ctrl, '/': search, Scroll desc: J(↓) K(↑) H(⇡), g/G: top/bot";
        }

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, _text_render_footer);
        self.render_list(list_area, buf, &render_list_title, &self.titles_library.clone(), &mut self.list_state_library.clone(), None);
        if !&self.titles_library.is_empty() {
            self.render_info_library(item_area1, buf, &self.list_state_library.clone());
            self.render_desc_library(item_area2, buf, &self.list_state_library.clone());
        }
    }

    /// `AppView::Settings` rendering
    fn render_settings(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);

        let render_list_title = "Settings";

        let mut _text_render_footer = "";
        if self.list_state_settings.selected() == Some(3) {
            // for `About` section
            _text_render_footer = "j/↓, k/↑: move, Scroll what's new: J(down) K(up) H(top),\n Tab: home, R: refresh, Q/Esc: quit.";
        }
        else if self.list_state_settings.selected() == Some(4) {
            _text_render_footer = "j/↓, k/↑: move, Scroll : J(down) K(up) H(top),\n Tab: home, R: refresh, Q/Esc: quit.";

        } else {
            _text_render_footer = "j/↓, k/↑: move, l/→: see options,\n Tab: home, R: refresh, Q/Esc: quit.";
        }

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, _text_render_footer);
        self.render_list(list_area, buf, render_list_title, &self.settings.clone(), &mut self.list_state_settings.clone(), None);
        self.render_info_settings(item_area1, buf, &self.list_state_settings.clone());
        self.render_desc_settings(item_area2, buf, &self.list_state_settings.clone());
    }

    /// `AppView::SettingsAccount` rendering
    fn render_settings_account(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, _item_area] = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1),]).areas(main_area);

        let render_list_title = "Settings account";
        let text_render_footer = "h: back, l/→: remove saved user,\n Tab: home, R: refresh, Q/Esc: quit.";

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(list_area, buf, render_list_title, &self.all_usernames.clone(), &mut self.list_state_settings_account.clone(), None);
        //self.render_selected_item(item_area, buf, &self.titles_library.clone(), self.auth_names_library.clone());
    }

    /// `AppView::SettingsLibrary` rendering
    fn render_settings_library(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area] = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1),]).areas(main_area);

        let items_number = self.libraries_names.len();
        let render_list_title = format!("Settings Library [{items_number} items]");

        let text_render_footer = "h: back, l/→: change library,\n Tab: home, R: refresh, Q/Esc: quit.";

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(list_area, buf, &render_list_title, &self.libraries_names.clone(), &mut self.list_state_settings_library.clone(), None);
        self.render_info_settings_library(item_area, buf, &self.list_state_settings_library.clone());
    }

    /// `AppView::SettingsAutoplay` rendering
    fn render_settings_autoplay(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area] = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1),]).areas(main_area);

        let render_list_title = "Podcast Autoplay";
        let text_render_footer = "h: back, l/→: apply,\n Tab: home, R: refresh, Q/Esc: quit.";
        let options = vec!["On".to_string(), "Off".to_string()];
        let current = if get_is_podcast_autoplay(&self.username) == "1" { "On" } else { "Off" };

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);
        self.render_list(list_area, buf, render_list_title, &options, &mut self.list_state_settings_autoplay.clone(), None);
        Paragraph::new(format!("Currently: {current}\n\nWhen on, finishing a podcast episode automatically starts the next unfinished one in the list it was played from."))
            .left_aligned()
            .wrap(Wrap { trim: true })
            .render(item_area, buf);
    }


    /// `AppView::SearchBook` rendering
    fn render_search_book(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);

        let render_list_title = "Search result";
        let mut _text_render_footer = "";
        if self.is_podcast {
        _text_render_footer = "j/↓, k/↑: move, l/→: episodes, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n '/': search, Scroll desc: J(down) K(up) H(top), g/G: top/bottom";
        } else {
        _text_render_footer = "j/↓, k/↑: move, l/→: play, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n '/': search, Scroll desc: J(down) K(up) H(top), g/G: top/bottom";
        } 


        if self.search_mode
            && let Ok(query) = self.search_active() {
                self.search_query = query.clone();
                self.search_mode = false; 
            }

        // init variables for search result (search by a book by title)
        let idx_and_titles: Vec<(usize, String)> = self.titles_library
            .iter()
            .enumerate() 
            .filter(|(_, x)| x.to_lowercase().contains(&self.search_query.to_lowercase())) 
            .map(|(index, title)| (index, title.clone())) 
            .collect();

        let mut titles_search_book_or_pod: Vec<String> = Vec::new();
        let mut index_to_keep: Vec<usize> = Vec::new();
        for (index, title) in idx_and_titles {
            titles_search_book_or_pod.push(title.clone());
            index_to_keep.push(index);
        }

        let titles_search_book_or_pod: &[String] = &titles_search_book_or_pod;

        // apply search filtering for book
        self.ids_search_book = self.ids_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.auth_names_pod_search_book = self.auth_names_library_pod
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.auth_names_search_book = self.auth_names_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.published_year_library_search_book = self.published_year_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.desc_library_search_book = self.desc_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.duration_library_search_book = self.duration_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| *value)
            .collect();
//        self.book_progress_search_book = self.book_progress_library
//            .iter()
//            .enumerate()
//            .filter(|(index, _)| index_to_keep.contains(&index))
//            .map(|(_, value)| value.clone())
//            .collect();
//        self.book_progress_search_book_cur_time = self.book_progress_library_cur_time
//            .iter()
//            .enumerate()
//            .filter(|(index, _)| index_to_keep.contains(&index))
//            .map(|(_, value)| value.clone())
//            .collect();
//        self.book_progress_search_book = self.book_progress_library
//            .iter()
//            .enumerate()
//            .filter(|(index, _)| index_to_keep.contains(&index))
//            .map(|(_, value)| value.clone())
//            .collect();

        // apply search filtering for podacst
        self.all_titles_pod_ep_search = self.all_titles_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_ids_pod_ep_search = self.all_ids_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_subtitles_pod_ep_search = self.all_subtitles_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_seasons_pod_ep_search = self.all_seasons_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_episodes_pod_ep_search = self.all_episodes_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_authors_pod_ep_search = self.all_authors_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_descs_pod_ep_search = self.all_descs_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_titles_pod_search = self.all_titles_pod
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.all_durations_pod_ep_search = self.all_durations_pod_ep
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();
        self.ids_library_pod_search = self.ids_library
            .iter()
            .enumerate()
            .filter(|(index, _)| index_to_keep.contains(index))
            .map(|(_, value)| value.clone())
            .collect();

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, _text_render_footer);
        self.render_list(list_area, buf, render_list_title, titles_search_book_or_pod, &mut self.list_state_search_results.clone(), None);
        if !titles_search_book_or_pod.is_empty() {
            self.render_info_search_book(item_area1, buf, &self.list_state_search_results.clone() );
            self.render_desc_search_book(item_area2, buf, &self.list_state_search_results.clone() );
        }
    }

    /// `AppView::PodcastEpisode`
    fn render_pod_ep(&mut self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, _player_area, _refresh_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(2),
        ]).areas(area);

        let [list_area, item_area1, item_area2] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(main_area);


        let text_render_footer = "j/↓, k/↑: move, l/→: play, h: back, Tab: home, R: refresh, S: Settings, Q/Esc: quit\n '/': search, Scroll desc: J(down) K(up) H(top), g/G: top/bottom";

        App::render_header(header_area, buf, self.lib_name_type.clone(), &self.username, &self.server_address_pretty, VERSION, &self.update_msg);
        App::render_footer(footer_area, buf, text_render_footer);
        let no_episodes_message = "No episodes found for this podcast.\nPress 'h' to go back.";

        if self.is_from_search_pod {
            if self.titles_pod_ep_search.is_empty() {
                log::warn!("render_pod_ep (search): No episodes found.");
                Paragraph::new(no_episodes_message)
                    .centered()
                    .block(Block::new().borders(Borders::TOP).border_style(Style::new().fg(Color::DarkGray)))
                    .render(main_area, buf);
            } else {
                let items_number = self.titles_pod_ep_search.len();
                let render_list_title = format!("Episodes [{items_number} items]");
                // Only render list/info/desc if episodes exist
                self.render_list(list_area, buf, &render_list_title, &self.titles_pod_ep_search.clone(), &mut self.list_state_pod_ep.clone(), None);
                self.render_info_pod_ep_search(item_area1, buf, &self.list_state_pod_ep.clone() );
                self.render_desc_pod_ep_search(item_area2, buf, &self.list_state_pod_ep.clone() );
            }
        } else {
            if self.titles_pod_ep.is_empty() {
                log::warn!("render_pod_ep (library): No episodes found.");
                Paragraph::new(no_episodes_message)
                    .centered()
                    .block(Block::new().borders(Borders::TOP).border_style(Style::new().fg(Color::DarkGray)))
                    .render(main_area, buf);
            } else {
                let items_number = self.titles_pod_ep.len();
                let render_list_title = format!("Episodes [{items_number} items]");
                // Only render list/info/desc if episodes exist
                self.render_list(list_area, buf, &render_list_title, &self.titles_pod_ep.clone(), &mut self.list_state_pod_ep.clone(), None);
                self.render_info_pod_ep(item_area1, buf, &self.list_state_pod_ep.clone() );
                self.render_desc_pod_ep(item_area2, buf, &self.list_state_pod_ep.clone() );
            }
        }
    }

    // General functions for rendering 

    fn render_header(area: Rect, buf: &mut Buffer, library_name: String, username: &str, server_address_pretty: &str, version: &str, update_msg: &str) {
        Paragraph::new(library_name)
            .bold()
            .centered()
            .render(area, buf);
        Paragraph::new(format!("👋 Connected as {username}\n🔗 {server_address_pretty}"))
            .not_bold()
            .left_aligned()
            .render(area, buf);
        Paragraph::new(format!("🦜 Absotui v{version}\n {update_msg}"))
            .right_aligned()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer, text_render_footer: &str) {
        Paragraph::new(text_render_footer)
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer, render_list_title: &str, render_list_items: &[String], list_state: &mut ListState, progress_info: Option<&[(String, f32, bool)]>) {
        let bg_color_header = self.config.colors.header_background_color.clone();
        let fg_color_header = self.config.colors.line_header_color.clone();
        let bg_color_block = self.config.colors.list_background_color.clone();
        let progress_bar_color = self.config.colors.progress_bar_color.clone();
        let progress_color = Color::Rgb(progress_bar_color[0], progress_bar_color[1], progress_bar_color[2]);
        // Deliberately no fg/bg/modifiers here at all - any of those get patched across
        // every cell in the row, overriding the row's own colors (the now-playing
        // marker's background, the progress underline). Selection is shown purely via
        // the highlight_symbol (a vertical bar) below, leaving the row itself untouched.
        let selected_style: Style = Style::default();

        let header_style: Style = Style::new()
            .fg(Color::Rgb(fg_color_header[0], fg_color_header[1], fg_color_header[2]))
            .bg(Color::Rgb(bg_color_header[0], bg_color_header[1], bg_color_header[2]));

        let block = Block::new()
            .title(Line::raw(render_list_title.to_string()).centered())
            .borders(Borders::TOP)
            .border_style(header_style)
            .bg(Color::Rgb(bg_color_block[0], bg_color_block[1], bg_color_block[2]));

        // Approximate content width available inside each row, after the "▎" highlight
        // symbol column that HighlightSpacing::Always reserves on every row.
        let content_width = area.width.saturating_sub(1) as usize;

        // Minimum gap (in characters) always kept clear between a title and the
        // time/age label, so a long title can never push the label off the row -
        // it gets truncated (or, on the selected row, scrolled) instead. Kept small
        // since the podcast age label already reserves its own trailing space via
        // its fixed-width left-alignment (see AGE_LABEL_WIDTH in render_home) - this
        // is just a little breathing room on top of that, and the only gap at all
        // for the book list's variable-length progress text.
        const MIN_TITLE_GAP: usize = 2;
        // How many ticks the marquee scroll holds still at the start/end of a
        // truncated title before continuing - purely a readability pause.
        const SCROLL_PAUSE_TICKS: u32 = 3;

        // Advance the title-scroll tick once per render (not once per row), on a
        // timer independent of render rate, and reset it whenever the selection
        // moves to a different row.
        let selected = list_state.selected();
        if selected != self.title_scroll_selected {
            self.title_scroll_selected = selected;
            self.title_scroll_offset = 0;
            self.title_scroll_last_tick = std::time::Instant::now();
        } else if self.title_scroll_last_tick.elapsed() >= std::time::Duration::from_millis(300) {
            self.title_scroll_offset = self.title_scroll_offset.wrapping_add(1);
            self.title_scroll_last_tick = std::time::Instant::now();
        }
        let scroll_offset = self.title_scroll_offset;

        let items: Vec<ListItem> = render_list_items
            .iter()
            .enumerate()
            .map(|(i, title)| {
                let color = Self::alternate_colors(i);
                match progress_info.and_then(|p| p.get(i)) {
                    Some((progress_text, percent, is_now_playing)) => {
                        // Line 1: now-playing marker (cobalt/progress-colored background) +
                        // title on the left, time/duration right-justified.
                        //
                        // The colored box itself is 3 columns wide with the ▶ glyph in the
                        // middle column, so the icon sits centered within its own box. A
                        // separate plain (uncolored) 1-column gap follows the box before
                        // the title, matching the 1-column blank the selection highlight
                        // symbol ("▎ ") already leaves before the box - so the box as a
                        // whole ends up with equal blank space on both sides of it too.
                        const MARKER_BOX_WIDTH: usize = 3;
                        const MARKER_GAP_WIDTH: usize = 1;
                        const MARKER_TOTAL_WIDTH: usize = MARKER_BOX_WIDTH + MARKER_GAP_WIDTH;
                        let marker_span = if *is_now_playing {
                            Span::styled(" ▶ ", Style::default().bg(progress_color))
                        } else {
                            Span::raw("   ")
                        };
                        let time_len = progress_text.chars().count();
                        let available_for_title = content_width.saturating_sub(MARKER_TOTAL_WIDTH + time_len + MIN_TITLE_GAP);
                        let title_chars: Vec<char> = title.chars().collect();

                        let display_title: String = if title_chars.len() <= available_for_title {
                            title.clone()
                        } else if available_for_title == 0 {
                            String::new()
                        } else if selected == Some(i) {
                            // Selected + truncated: scroll a window across the title to
                            // reveal the hidden tail, pausing at both ends before looping.
                            let overflow = title_chars.len() - available_for_title;
                            let cycle_len = overflow as u32 + 2 * SCROLL_PAUSE_TICKS;
                            let pos = scroll_offset % cycle_len;
                            let window_start = if pos < SCROLL_PAUSE_TICKS {
                                0
                            } else if pos < SCROLL_PAUSE_TICKS + overflow as u32 {
                                (pos - SCROLL_PAUSE_TICKS) as usize
                            } else {
                                overflow
                            };
                            title_chars[window_start..window_start + available_for_title].iter().collect()
                        } else {
                            let cut = available_for_title.saturating_sub(1);
                            format!("{}…", title_chars[..cut].iter().collect::<String>())
                        };

                        let title_len = display_title.chars().count();
                        let padding = content_width.saturating_sub(MARKER_TOTAL_WIDTH + title_len + time_len);

                        // Progress shown as an underline beneath the time text itself -
                        // not a full-height background fill - filled up to percent complete.
                        let time_chars: Vec<char> = progress_text.chars().collect();
                        let fill_count = (((percent / 100.0) * time_chars.len() as f32).round() as usize).min(time_chars.len());
                        let time_filled: String = time_chars[..fill_count].iter().collect();
                        let time_unfilled: String = time_chars[fill_count..].iter().collect();

                        let line1 = Line::from(vec![
                            marker_span,
                            Span::raw(" ".repeat(MARKER_GAP_WIDTH)),
                            Span::raw(display_title),
                            Span::raw(" ".repeat(padding)),
                            Span::styled(time_filled, Style::default().underline_color(progress_color).add_modifier(Modifier::UNDERLINED)),
                            Span::raw(time_unfilled),
                        ]);

                        ListItem::new(line1).bg(color)
                    }
                    None => ListItem::new(title.clone()).bg(color),
                }
            })
        .collect();


        let list = List::new(items)
            .block(block)
            .highlight_style(selected_style)
            .highlight_symbol("▎")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, list_state);
    }


    // info about the book or podacst for `Home`
    fn render_info_home(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        let duration_cnt_list_conv = convert_seconds(self.duration_cnt_list.clone());

        // Chapter rows don't have their own info to show - resolve back to the book they
        // belong to. Cursor position no longer maps 1:1 to a book index once chapter rows
        // are spliced in, so this has to go through the same row-building the list itself
        // used, or it reads (or panics on) the wrong entry.
        let selected = if self.is_podcast {
            list_state.selected()
        } else {
            list_state.selected().and_then(|i| self.build_home_rows().get(i).map(|row| match row {
                HomeRow::Book(book_i) => *book_i,
                HomeRow::Chapter { book_index, .. } => *book_index,
            }))
        };

        if let Some(selected) = selected {

            if self.is_podcast {
                Paragraph::new(format!("[{}] - Author: {} - Episode: {} - Duration: {}", 
                        self.titles_pod_cnt_list[selected], 
                        self.authors_pod_cnt_list[selected], 
                        self.nums_ep_pod_cnt_list[selected],
                        self.durations_pod_cnt_list[selected],
                ))
                    .left_aligned()
                    .render(area, buf);
                } else {
                    Paragraph::new(format!("Author: {} - Year: {} - Duration: {}\nProgress: {}%, {} {}", 
                            self.auth_names_cnt_list[selected], 
                            self.pub_year_cnt_list[selected], 
                            duration_cnt_list_conv[selected],
                            self.book_progress_cnt_list[selected][0], // percentage progression
                            convert_seconds_for_prg(self.duration_cnt_list[selected], self.book_progress_cnt_list_cur_time[selected][0]), // time left
                            self.book_progress_cnt_list[selected][1], // is finished
                    ))
                        .left_aligned()
                        .render(area, buf);
            }
        }
    }

    // description of the book or podcast `Home`
    fn render_desc_home(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        // See render_info_home - chapter rows resolve back to their parent book's index.
        let selected = if self.is_podcast {
            list_state.selected()
        } else {
            list_state.selected().and_then(|i| self.build_home_rows().get(i).map(|row| match row {
                HomeRow::Book(book_i) => *book_i,
                HomeRow::Chapter { book_index, .. } => *book_index,
            }))
        };

        if let Some(selected) = selected {
            let mut _content: String = String::new();
            if self.is_podcast {
                _content = self.subtitles_pod_cnt_list[selected].clone();
            } else {
                _content = self.desc_cnt_list[selected].clone();
            }

            Paragraph::new(_content.clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
            }
    }

    // info about the book or podacst for `Library`
    fn render_info_library(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        let _duration_library_conv = convert_seconds(self.duration_library.clone());

        if let Some(selected) = list_state.selected() {
            if self.is_podcast {
                Paragraph::new(format!("Author: {}", 
                        self.auth_names_library_pod[selected], 
                ))
                    .left_aligned()
                    .render(area, buf);
            } 
            else {
                Paragraph::new(format!("Author: {} - Year: {}", //- Duration: {}\nProgress:{} {}{}", 
                        self.auth_names_library[selected], 
                        self.published_year_library[selected], 

                        //duration_library_conv[selected],
                        //self.book_progress_library[selected][0], // percentage progression
                        //format!("{}",convert_seconds_for_prg(self.duration_library[selected], self.book_progress_library_cur_time[selected][0])), // time left
                        //self.book_progress_library[selected][1] // is_finished
                        )) 
                    .left_aligned()
                    .render(area, buf);
            }
        }
    }

    // description of the book or podcast `Library`
    fn render_desc_library(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {

            Paragraph::new(self.desc_library[selected].clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
        }
    }

    // info about the podcast for `PodcastEpisode`
    fn render_info_pod_ep(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        // Check if source vectors for podcast title/author are empty before accessing index 0
        if self.titles_pod.is_empty() || self.authors_pod_ep.is_empty() {
            log::error!("render_info_pod_ep: titles_pod or authors_pod_ep is empty. Cannot render episode info.");
            // Render placeholder text or handle appropriately
            Paragraph::new("Error: Podcast metadata missing.")
                .left_aligned()
                .render(area, buf);
            return; // Exit the function early
        }

        let n = self.durations_pod_ep.len();
        // Now safe to access index 0 as we've checked they are not empty
        let duplicated_titles = vec![self.titles_pod[0].clone(); n];
        let duplicated_authors = vec![self.authors_pod_ep[0].clone(); n];

        if let Some(selected) = list_state.selected() {
            log::debug!(
                "render_info_pod_ep: selected={}, titles_pod.len={}, authors_pod_ep.len={}, durations_pod_ep.len={}, episodes_pod_ep.len={}, duplicated_titles.len={}, duplicated_authors.len={}",
                selected,
                self.titles_pod.len(), // Should be >= 1 here
                self.authors_pod_ep.len(), // Should be >= 1 here
                self.durations_pod_ep.len(),
                self.episodes_pod_ep.len(),
                duplicated_titles.len(), // Will be n
                duplicated_authors.len() // Will be n
            );

            // Check if episode-specific vectors are valid for the selected index
            if selected < self.episodes_pod_ep.len() && selected < self.durations_pod_ep.len() {
                 // Also check duplicated vectors, though their length depends on n (durations_pod_ep.len())
                 if selected < duplicated_titles.len() && selected < duplicated_authors.len() {
                    Paragraph::new(format!("[{}] - Author: {} - Episode: {} - Duration: {} ",
                            duplicated_titles[selected].trim(),
                            duplicated_authors[selected].trim(),
                            self.episodes_pod_ep[selected].trim(),
                            self.durations_pod_ep[selected].trim(),
                    ))
                        .left_aligned()
                        .render(area, buf);
                 } else {
                     log::error!("render_info_pod_ep: Index {} out of bounds for duplicated title/author vectors (len={})!", selected, duplicated_titles.len());
                     Paragraph::new("Error: Episode info rendering mismatch.")
                         .left_aligned()
                         .render(area, buf);
                 }
            } else {
                log::error!("render_info_pod_ep: Index {} out of bounds for episode/duration vectors (ep_len={}, dur_len={})!", selected, self.episodes_pod_ep.len(), self.durations_pod_ep.len());
                Paragraph::new("Error: Episode data unavailable or index out of bounds.")
                    .left_aligned()
                    .render(area, buf);
            }
        }
    }
    // info about the podcast for `PodcastEpisode` (from search)
    fn render_info_pod_ep_search(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        let n = self.durations_pod_ep_search.len();
        let duplicated_titles_search = vec![self.titles_pod_search[0].clone(); n];
        let duplicated_authors_search = vec![self.authors_pod_ep_search[0].clone(); n];
        if let Some(selected) = list_state.selected() {

            Paragraph::new(format!("[{}] - Author: {} - Episode: {} - Duration: {} ", 
                    duplicated_titles_search[selected].trim(), 
                    duplicated_authors_search[selected].trim(), 
                    self.episodes_pod_ep_search[selected].trim(),
                    self.durations_pod_ep_search[selected].trim(),
            ))
                .left_aligned()
                .render(area, buf);
        }
    }

    // desc of the podcast for `PodcastEpisode`
    fn render_desc_pod_ep(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {
            log::debug!("render_desc_pod_ep: selected={}, subtitles_pod_ep.len={}", selected, self.subtitles_pod_ep.len());

            // Check if index is valid for subtitles vector
            if selected < self.subtitles_pod_ep.len() {
                Paragraph::new(self.subtitles_pod_ep[selected].clone())
                    .scroll((self.scroll_offset, 0))
                    .wrap(Wrap { trim: true })
                    .render(area, buf);
            } else {
                log::error!("render_desc_pod_ep: Index {} out of bounds for subtitles_pod_ep (len={})!", selected, self.subtitles_pod_ep.len());
                // Render placeholder text
                Paragraph::new("Error: Episode description unavailable.")
                    .left_aligned()
                    .render(area, buf);
            }
        }
    }
    // desc of the podcast for `PodcastEpisode` (from search)
    fn render_desc_pod_ep_search(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {

            Paragraph::new(self.subtitles_pod_ep_search[selected].clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
        }
    }

    // info about the book or podacst for `SearchBook`
    fn render_info_search_book(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {
        let _duration_library_search_book_conv = convert_seconds(self.duration_library_search_book.clone());

        if let Some(selected) = list_state.selected() {
            if self.is_podcast {
                Paragraph::new(format!("Author: {}", 
                        self.auth_names_pod_search_book[selected], 
                ))
                    .left_aligned()
                    .render(area, buf);
            } 
            else {
                Paragraph::new(format!("Author: {} - Year: {}", //- Duration: {}\nProgress:{} {}{}", 
                        self.auth_names_search_book[selected], 
                        self.published_year_library_search_book[selected], 
                      //  duration_library_search_book_conv[selected],
                      //  self.book_progress_search_book[selected][0], // percentage progression
                      //  format!("{}",convert_seconds_for_prg(self.duration_library_search_book[selected], self.book_progress_search_book_cur_time[selected][0])), // time left
                      //  self.book_progress_search_book[selected][1] // is finished
                        )) 
                    .left_aligned()
                    .render(area, buf);
            }
        }
    }

    // description of the book or podcast `SearchBook`
    fn render_desc_search_book(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {

            Paragraph::new(self.desc_library_search_book[selected].clone())
                .scroll((self.scroll_offset, 0))
                .wrap(Wrap { trim: true })
                .render(area, buf);
        }
    }

    // info for settings
    fn render_info_settings(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        match list_state.selected() {
            Some(0) => {}
            Some(1) => {}
            Some(2) => {}
            Some(3) => {

                Paragraph::new(format!("Absotui v{} - Licence: GPL-3.0 - Issues: {}/issues\nSource code: {}\nWhat's new:",
                        VERSION,
                        "https://github.com/pdwaldrop/Absotui",
                        "https://github.com/pdwaldrop/Absotui",
                ))
                    .left_aligned()
                    .render(area, buf);
                }
            _ => {}
        }

    }

    
    // desc for settings
    fn render_desc_settings(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        let instructions = "\
Update:
- Quit the app
- If you built from source: git pull && cargo build --release
- If you installed using the script: absotui --update

Uninstall:
- Quit the app
- If you built from source: just delete the cloned folder
- If you installed using the script: absotui --uninstall
";

        match list_state.selected() {

            Some(0) => {}
            Some(1) => {}
            Some(2) => {}
            Some(3) => {
                Paragraph::new(self.changelog.clone())
                    .scroll((self.scroll_offset, 0))
                    .wrap(Wrap { trim: true })
                    .render(area, buf);
                }
            Some(4) => {
                Paragraph::new(instructions)
                    .scroll((self.scroll_offset, 0))
                    .wrap(Wrap { trim: true })
                    .render(area, buf);
                }
            _ =>  {}
        }
    }

    // info for settings library
    fn render_info_settings_library(&self, area: Rect, buf: &mut Buffer, list_state: &ListState) {

        if let Some(selected) = list_state.selected() {
                Paragraph::new(format!("Type: {}", 
                        self.media_types[selected], 
                ))
                    .left_aligned()
                    .render(area, buf);
            } 

    }

    fn alternate_colors(i: usize) -> Color {
        let mut color_bg_list = Vec::new();
        let mut color_alt_bg_list = Vec::new();
        if let Ok(cfg) = load_config() {
            color_bg_list = cfg.colors.list_background_color;
            color_alt_bg_list = cfg.colors.list_background_color_alt_row;
        }
        if i.is_multiple_of(2) {
            Color::Rgb(color_bg_list[0], color_bg_list[1], color_bg_list[2])
        } else {
            Color::Rgb(color_alt_bg_list[0], color_alt_bg_list[1], color_alt_bg_list[2])
        }
    }
}
