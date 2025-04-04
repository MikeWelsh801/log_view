use color_eyre::Result;
use std::{env, process};

use log_viewer::{Config, run};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|error| {
        eprintln!("Couldn't parse args: {error}");
        process::exit(1);
    });

    run(config)
}
