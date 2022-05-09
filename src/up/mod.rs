use config::Services;
use crate::command::Command;
use std::{fs::File, path::Path};

mod config;

pub fn command<'args>() -> Command<'args> {
    Command{
        name: "up",
        specs: |name| { clap::Command::new(name).about("Start all services") },
        run: |args| { Box::pin(run(args)) }
    }
}

async fn run(_: &clap::ArgMatches) -> Result<(), String> {
    let config = load_config("locenv-services.yml")?;

    Ok(())
}

fn load_config<P: AsRef<Path>>(path: P) -> Result<Services, String> {
    let file = match File::open(&path) {
        Ok(r) => r,
        Err(e) => return Err(format!("Failed to open {}: {}", path.as_ref().display(), e)),
    };

    let config = match Services::load(file) {
        Ok(r) => r,
        Err(e) => return Err(format!("{}: {}", path.as_ref().display(), e)),
    };

    Ok(config)
}
