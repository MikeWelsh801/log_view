use rust_fuzzy_search::fuzzy_search_threshold;
use std::fs;

use crate::{Config, Message};
use color_eyre::Result;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum Filter {
    INFO,
    WARNING,
    ERROR,
    CRITICAL,
    DEBUG,
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
    g_modifier: bool,
    pub(crate) search_mode: SearchMode,
    pub(crate) search_input: String,
    pub(crate) cursor_pos: usize,
    pub(crate) log_path: String,
    pub(crate) log_filter: Filter,
    pub(crate) running: RunningState,
    logs: Vec<String>,
}

impl Model {
    pub(crate) fn new(config: Config) -> Result<Model> {
        let mut model = Model {
            view_offset: 0,
            view_height: 0,
            g_modifier: false,
            search_mode: SearchMode::default(),
            search_input: String::new(),
            cursor_pos: 0,
            log_path: config.file_path.clone(),
            log_filter: Filter::NONE,
            running: RunningState::default(),
            logs: vec![],
        };

        model.refresh_logs();
        Ok(model)
    }

    pub(crate) fn set_view_height(&mut self, height: usize) {
        self.view_height = height;
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.search_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_pos)
            .unwrap_or(self.search_input.len())
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.search_input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.cursor_pos = 0;
    }

    fn refresh_logs(&mut self) {
        let logs: Vec<String> = fs::read_to_string(&self.log_path)
            .unwrap_or(String::new())
            .lines()
            .map(|l| l.to_string())
            .collect();

        // If the we've added logs and we're not at the bottom of the view,
        // compensate the view offset so the filtered view doesn't scroll us
        // downward when adding logs.
        if logs.len() > self.logs.len() && self.view_offset != 0 {
            self.view_offset += logs.len() - self.logs.len();
        }

        self.logs = logs;
    }
}

/************************ Search Input Functions *****************************/
fn enter_char(model: &mut Model, new_char: char) {
    let index = model.byte_index();
    model.search_input.insert(index, new_char);
    move_cursor_right(model);
}

fn move_cursor_left(model: &mut Model) {
    let cursor_moved_left = model.cursor_pos.saturating_sub(1);
    model.cursor_pos = model.clamp_cursor(cursor_moved_left);
}

fn move_cursor_right(model: &mut Model) {
    let cursor_moved_right = model.cursor_pos.saturating_add(1);
    model.cursor_pos = model.clamp_cursor(cursor_moved_right);
}

fn delete_char(model: &mut Model) {
    let is_not_cursor_leftmost = model.cursor_pos != 0;
    if is_not_cursor_leftmost {
        // Method "remove" is not used on the saved text for deleting the selected char.
        // Reason: Using remove on String works on bytes instead of the chars.
        // Using remove would require special care because of char boundaries.

        let current_index = model.cursor_pos;
        let from_left_to_current_index = current_index - 1;

        // Getting all characters before the selected character.
        let before_char_to_delete = model.search_input.chars().take(from_left_to_current_index);
        // Getting all characters after selected character.
        let after_char_to_delete = model.search_input.chars().skip(current_index);

        // Put all characters together except the selected one.
        // By leaving the selected one out, it is forgotten and therefore deleted.
        model.search_input = before_char_to_delete.chain(after_char_to_delete).collect();
        move_cursor_left(model);
    }
}

fn reset_search(model: &mut Model) {
    model.search_input.clear();
    model.reset_cursor();
    model.search_mode = SearchMode::None;
}

/*****************************************************************************/

pub(crate) fn update(model: &mut Model, msg: Message) -> Option<Message> {
    if model.g_modifier {
        match msg {
            Message::MoveTop => {
                model.view_offset = 0xffff;
                return None;
            }
            _ => model.g_modifier = false,
        };
    }

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
            model.view_offset = 0;
        }
        Message::Quit => {
            model.running = RunningState::Done;
        }
        Message::ToggleSearch => match model.search_mode {
            SearchMode::Search => {
                reset_search(model);
            }
            SearchMode::None => {
                model.search_mode = SearchMode::Search;
            }
        },
        Message::AddChar(c) => enter_char(model, c),
        Message::Delete => delete_char(model),
        Message::MoveCursorLeft => move_cursor_left(model),
        Message::MoveCursorRight => move_cursor_right(model),
        Message::RefreshLogs => model.refresh_logs(),
        Message::MoveTop => model.g_modifier = true,
        Message::MoveBottom => model.view_offset = 0,
    };
    None
}

pub(crate) fn get_filtered_logs(model: &mut Model) -> Vec<String> {
    let filter_str = match model.log_filter {
        Filter::INFO => "INFO",
        Filter::WARNING => "WARNING",
        Filter::ERROR => "ERROR",
        Filter::CRITICAL => "CRITICAL",
        Filter::DEBUG => "DEBUG",
        Filter::NONE | Filter::SELECT => "",
    };

    let mut logs = model
        .logs
        .iter()
        .filter(|line| line.contains(filter_str))
        .map(|l| l.to_string())
        .collect::<Vec<String>>();

    match apply_search(model, &mut logs) {
        true => logs,
        false => {
            if model.view_offset + model.view_height > logs.len() {
                model.view_offset = logs.len().checked_sub(model.view_height).unwrap_or(0);
            }

            let end_idx = logs
                .len()
                .checked_sub(model.view_offset)
                .unwrap_or(model.view_height);
            let start_idx = end_idx.checked_sub(model.view_height).unwrap_or(0);
            logs.drain(start_idx..end_idx).collect()
        }
    }
}

fn apply_search(model: &mut Model, logs: &mut Vec<String>) -> bool {
    if !model.search_input.is_empty() {
        let search_logs: Vec<&str> = logs.iter().map(|log| log.as_str()).collect();

        *logs = fuzzy_search_threshold(&model.search_input, &search_logs, 0.4)
            .iter()
            .map(|res| res.0.to_string())
            .rev()
            .collect();
        return true;
    };
    false
}
