use crate::{Filter, Message, Model, SearchMode, get_filtered_logs};
use color_eyre::eyre::Ok;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};
use std::time::Duration;
use strip_ansi_escapes::strip;

pub(crate) fn view(frame: &mut Frame, model: &mut Model) {
    let opts_height = 3;
    let filter_height = 1;

    let [log_area, search_area, opts_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Length(opts_height),
            Constraint::Length(filter_height),
        ])
        .areas(frame.area());

    model.set_view_height((log_area.height - 2) as usize);

    let [log_list, log_preview] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .areas(log_area);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title("logs")
        .title_alignment(Alignment::Center);

    let filtered_logs = get_filtered_logs(model);

    let lines = filtered_logs
        .iter()
        .enumerate()
        .map(|(idx, l)| get_formatted_row(l, model.line_idx == idx))
        .collect();

    let default = String::new();
    let curr_log = filtered_logs.get(model.line_idx).unwrap_or(&default);
    let preview_paragraph = Paragraph::new(String::from_utf8(strip(curr_log.as_bytes())).unwrap())
        .wrap(Wrap { trim: false })
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("preview")
                .title_alignment(Alignment::Center),
        );

    let line_paragraph = Table::from(lines).block(block);

    let search = Paragraph::new(model.search_input.as_str())
        .style(match model.search_mode {
            SearchMode::None => Style::default(),
            SearchMode::Search => Style::default().fg(Color::Cyan),
        })
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("search"),
        );

    render_opts(model, frame, opts_area);
    frame.render_widget(line_paragraph, log_list);
    frame.render_widget(preview_paragraph, log_preview);
    frame.render_widget(search, search_area);

    set_cursor_pos(model, frame, search_area);
}

pub(crate) fn handle_event(m: &mut Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(400))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key, m));
            }
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent, model: &mut Model) -> Option<Message> {
    if model.search_mode == SearchMode::Search {
        return match key.code {
            KeyCode::Enter | KeyCode::Esc => Some(Message::ToggleSearch),
            // Ctrl-c can exit search mode
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Message::ToggleSearch)
            }
            KeyCode::Char(insert_char) => Some(Message::AddChar(insert_char)),
            KeyCode::Backspace => Some(Message::Delete),
            KeyCode::Left => Some(Message::MoveCursorLeft),
            KeyCode::Right => Some(Message::MoveCursorRight),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::MoveDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::MoveUp),
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('g') => Some(Message::MoveTop),
        KeyCode::Char('G') => Some(Message::MoveBottom),
        KeyCode::Char('s') | KeyCode::Char('/') => Some(Message::ToggleSearch),
        KeyCode::Char('f') => {
            if model.log_filter == Filter::SELECT {
                Some(Message::ApplyFilter(Filter::NONE))
            } else {
                Some(Message::ApplyFilter(Filter::SELECT))
            }
        }
        KeyCode::Char('i') => {
            if model.log_filter == Filter::SELECT {
                Some(Message::ApplyFilter(Filter::INFO))
            } else {
                None
            }
        }
        KeyCode::Char('w') => {
            if model.log_filter == Filter::SELECT {
                Some(Message::ApplyFilter(Filter::WARNING))
            } else {
                None
            }
        }
        KeyCode::Char('e') => {
            if model.log_filter == Filter::SELECT {
                Some(Message::ApplyFilter(Filter::ERROR))
            } else {
                None
            }
        }
        KeyCode::Char('c') => {
            if model.log_filter == Filter::SELECT {
                Some(Message::ApplyFilter(Filter::CRITICAL))
            } else {
                None
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::MoveUpPage)
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::MoveDownPage)
        }
        KeyCode::Char('d') => {
            if model.log_filter == Filter::SELECT {
                Some(Message::ApplyFilter(Filter::DEBUG))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn render_opts(model: &Model, frame: &mut Frame, opts_area: Rect) {
    if model.search_mode == SearchMode::Search {
        let opts = Table::default()
            .rows([Row::new(vec![" Exit Search: Esc/Ctrl-c"])])
            .cyan()
            .bold();
        frame.render_widget(opts, opts_area);
        return;
    }

    match model.log_filter {
        Filter::SELECT => {
            let opts = Table::default()
                .rows([Row::new(vec![
                    " quit: q",
                    "info: i",
                    "warning: w",
                    "error: e",
                    "critical: c",
                    "debug: d",
                ])])
                .cyan()
                .bold();
            frame.render_widget(opts, opts_area);
        }
        _ => {
            let opts = Table::default()
                .rows([Row::new(vec![" quit: q", "filter: f", "search: s or /"])])
                .cyan()
                .bold();
            frame.render_widget(opts, opts_area);
        }
    };
}

fn get_formatted_row(log: &String, current_log: bool) -> Row {
    if current_log {
        Row::new(vec![String::from_utf8(strip(log.as_bytes())).unwrap()])
            .black()
            .on_cyan()
    } else if log.contains("INFO") {
        Row::new(vec![String::from_utf8(strip(log.as_bytes())).unwrap()]).cyan()
    } else if log.contains("WARNING") {
        Row::new(vec![String::from_utf8(strip(log.as_bytes())).unwrap()]).yellow()
    } else if log.contains("ERROR") {
        Row::new(vec![String::from_utf8(strip(log.as_bytes())).unwrap()]).red()
    } else if log.contains("CRITICAL") {
        Row::new(vec![String::from_utf8(strip(log.as_bytes())).unwrap()])
            .bold()
            .black()
            .on_red()
    } else {
        Row::new(vec![String::from_utf8(strip(log.as_bytes())).unwrap()])
    }
}

fn set_cursor_pos(model: &mut Model, frame: &mut Frame, input_area: Rect) {
    match model.search_mode {
        #[allow(clippy::cast_possible_truncation)]
        SearchMode::Search => frame.set_cursor_position(Position::new(
            // Draw the cursor at the current position in the input field.
            // This position is can be controlled via the left and right arrow key
            input_area.x + model.cursor_pos as u16 + 1,
            // Move one line down, from the border to the input line
            input_area.y + 1,
        )),
        SearchMode::None => {}
    }
}
