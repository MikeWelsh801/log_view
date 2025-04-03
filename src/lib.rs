use color_eyre::Result;

pub mod messages;
pub mod model;
pub mod view;

pub(crate) use crate::messages::log_message::*;
pub(crate) use crate::model::log_model::*;
pub(crate) use crate::view::log_view::*;
pub(crate) use crate::view::tui;

pub struct Config {
    file_path: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Self, &'static str> {
        if args.len() < 2 {
            return Err("Must provide a file path.");
        }
        let file_path = args[1].clone();

        Ok(Config {
            file_path,
        })
    }
}

pub fn run(config: Config) -> Result<()> {
    tui::install_panic_hook();
    let mut terminal = tui::init_terminal()?;
    let mut model = Model::new(config)?;

    while model.running != RunningState::Done {
        // render the current view
        terminal.draw(|frame| view(frame, &mut model))?;

        let mut current_msg = handle_event(&mut model)?;

        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }
    }

    tui::restore_terminal()?;
    Ok(())
}
