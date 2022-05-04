use config::Services;
use crate::command::Command;

mod config;

pub fn command<'args>() -> Command<'args> {
    Command{
        name: "up",
        specs: |name| { clap::Command::new(name).about("Start all services") },
        run: |args| { Box::pin(run(args)) }
    }
}

async fn run(_: &clap::ArgMatches) -> Result<(), String> {
    let config = Services::load("locenv-services.yml")?;

    Ok(())
}
