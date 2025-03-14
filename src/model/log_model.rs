use rust_fuzzy_search::fuzzy_search_best_n;
use std::{cmp::max, fs};

use crate::{Config, Message};
use color_eyre::Result;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum Filter {
    INFO,
    WARNING,
    ERROR,
    CRITICAL,
    SELECT,
    #[default]
    NONE,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum SearchMode {
    Search,
    #[default]
    None,
}

#[derive(Debug, Default)]
pub(crate) struct Model {
    view_offset: usize,
    view_height: usize,
    pub(crate) search_mode: SearchMode,
    pub(crate) search_input: String,
    pub(crate) cursor_pos: usize,
    pub(crate) log_path: String,
    pub(crate) log_filter: Filter,
    pub(crate) running: RunningState,
    pub(crate) logs: Vec<String>,
}

impl Model {
    pub(crate) fn new(config: Config) -> Result<Model> {
        let mut model = Model {
            view_offset: 0,
            view_height: config.max_len,
            search_mode: SearchMode::default(),
            search_input: String::new(),
            cursor_pos: 0,
            log_path: config.file_path.clone(),
            log_filter: Filter::NONE,
            running: RunningState::default(),
            logs: vec![],
        };

        model.logs = filter_logs(&mut model);
        Ok(model)
    }

    pub(crate) fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_pos.saturating_sub(1);
        self.cursor_pos = self.clamp_cursor(cursor_moved_left);
    }

    pub(crate) fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_pos.saturating_add(1);
        self.cursor_pos = self.clamp_cursor(cursor_moved_right);
    }

    pub(crate) fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.search_input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    pub(crate) fn byte_index(&self) -> usize {
        self.search_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_pos)
            .unwrap_or(self.search_input.len())
    }

    pub(crate) fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_pos != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_pos;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.search_input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.search_input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.search_input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub(crate) fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.search_input.chars().count())
    }

    pub(crate) fn reset_cursor(&mut self) {
        self.cursor_pos = 0;
    }

    pub(crate) fn submit_message(&mut self) {
        let search_logs: Vec<&str> = self.logs.iter().map(|log| log.as_str()).collect();
        self.logs = fuzzy_search_best_n(&self.search_input, &search_logs, 30)
            .iter()
            .map(|res| res.0.to_string())
            .rev()
            .collect();

        self.search_input.clear();
        self.reset_cursor();
        self.search_mode = SearchMode::None; 
    }
}

pub(crate) fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::MoveUp => {
            model.view_offset += 1;
        }
        Message::MoveDown => {
            if model.view_offset > 0 {
                model.view_offset -= 1;
            }
        }
        Message::ApplyFilter(f) => {
            model.log_filter = f;
        }
        Message::Quit => {
            model.running = RunningState::Done;
        }
        Message::ToggleSearch => match model.search_mode {
            SearchMode::Search => {
                model.search_input.clear();
                model.cursor_pos = 0;
                model.search_mode = SearchMode::None;
            }
            SearchMode::None => {
                model.search_mode = SearchMode::Search;
            }
        },
    };
    model.logs = filter_logs(model);
    None
}

fn filter_logs(model: &mut Model) -> Vec<String> {
    let mut logs: Vec<String> = fs::read_to_string(&model.log_path)
        .unwrap_or(String::new())
        .lines()
        .map(|l| l.to_string())
        .collect();

    let end_idx = max(logs.len() - model.view_offset, 0);
    let start_idx = max(end_idx - model.view_height, 0);

    match model.log_filter {
        Filter::INFO => {
            let mut logs = logs
                .iter()
                .filter(|line| line.contains("INFO"))
                .map(|l| l.to_string())
                .collect::<Vec<String>>();

            let end_idx = logs.len().checked_sub(model.view_offset).unwrap_or(0);
            let start_idx = end_idx.checked_sub(model.view_height).unwrap_or(0);

            logs.drain(start_idx..end_idx).collect()
        }
        Filter::WARNING => {
            let mut logs = logs
                .iter()
                .filter(|line| line.contains("WARNING"))
                .map(|l| l.to_string())
                .collect::<Vec<String>>();

            let end_idx = logs.len().checked_sub(model.view_offset).unwrap_or(0);
            let start_idx = end_idx.checked_sub(model.view_height).unwrap_or(0);

            logs.drain(start_idx..end_idx).collect()
        }
        Filter::ERROR => {
            let mut logs = logs
                .iter()
                .filter(|line| line.contains("ERROR"))
                .map(|l| l.to_string())
                .collect::<Vec<String>>();

            let end_idx = logs.len().checked_sub(model.view_offset).unwrap_or(0);
            let start_idx = end_idx.checked_sub(model.view_height).unwrap_or(0);

            logs.drain(start_idx..end_idx).collect()
        }
        Filter::CRITICAL => {
            let mut logs = logs
                .iter()
                .filter(|line| line.contains("CRITICAL"))
                .map(|l| l.to_string())
                .collect::<Vec<String>>();

            let end_idx = logs.len().checked_sub(model.view_offset).unwrap_or(0);
            let start_idx = end_idx.checked_sub(model.view_height).unwrap_or(0);

            logs.drain(start_idx..end_idx).collect()
        }

        _ => logs.drain(start_idx..end_idx).collect(),
    }
}
