use crate::{Filter, Message, Model, SearchMode};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};
use std::time::Duration;
use strip_ansi_escapes::strip;

pub(crate) fn view(frame: &mut Frame, model: &mut Model) {
    let line_height = 88;
    let opts_height = 4;

    let [log_area, search_area, opts_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(line_height),
            Constraint::Percentage(100 - line_height - opts_height),
            Constraint::Percentage(opts_height),
        ])
        .areas(frame.area());

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title("logs")
        .title_alignment(Alignment::Center);

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

    let lines = model.logs.iter().map(|l| get_formatted_row(l)).collect();
    let line_paragraph = Table::from(lines).block(block);

    render_opts(model, frame, opts_area);
    frame.render_widget(line_paragraph, log_area);
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

fn get_formatted_row(log: &String) -> Row {
    if log.contains("INFO") {
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

fn handle_key(key: event::KeyEvent, model: &mut Model) -> Option<Message> {
    let ctrl_c = key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);

    if model.search_mode == SearchMode::Search && key.code != KeyCode::Esc && !ctrl_c {
        match key.code {
            KeyCode::Enter => model.submit_message(),
            KeyCode::Char(insert_char) => model.enter_char(insert_char),
            KeyCode::Backspace => model.delete_char(),
            KeyCode::Left => model.move_cursor_left(),
            KeyCode::Right => model.move_cursor_right(),
            _ => {}
        }
        return None;

    }

    if ctrl_c && model.search_mode == SearchMode::Search {
        return Some(Message::ToggleSearch);
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::MoveDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::MoveUp),
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('s') | KeyCode::Char('/') | KeyCode::Esc => Some(Message::ToggleSearch),
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
