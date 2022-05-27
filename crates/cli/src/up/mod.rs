use self::errors::{
    AlreadyRunning, ConfigOpenError, ConfigParseError, ServiceManagerPortFileOpenError,
};
use crate::command::Command;
use crate::context::Context;
use clap::ArgMatches;
use config::Services;
use std::error::Error;
use std::fs::File;
use std::path::Path;

mod config;
mod errors;
mod repository;

pub fn command<'args>() -> Command<'args> {
    Command {
        name: "up",
        specs: |name| clap::Command::new(name).about("Start all services"),
        run: |ctx, args| Box::pin(run(ctx, args)),
    }
}

async fn run(ctx: &Context, _: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // Load config.
    let conf = read_config(ctx.project().join("locenv-services.yml"))?;

    // Check if services already running.
    if is_running(ctx)? {
        return Err(AlreadyRunning::new().into());
    }

    // Update local repositories.
    for (n, s) in &conf {
        repository::update(ctx, n, &s.repository).await?;
    }

    Ok(())
}

fn is_running(ctx: &Context) -> Result<bool, Box<dyn Error>> {
    // Build path to the file containing service manager port.
    let mut port = ctx.service_manager();

    port.push("port");

    // Open the file.
    match File::open(&port) {
        Ok(r) => r,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => return Ok(false),
            _ => return Err(ServiceManagerPortFileOpenError::new(port, e).into()),
        },
    };

    Ok(true)
}

fn read_config<P: AsRef<Path>>(path: P) -> Result<Services, Box<dyn Error>> {
    let file = match File::open(&path) {
        Ok(r) => r,
        Err(e) => return Err(ConfigOpenError::new(path.as_ref(), e).into()),
    };

    let config = match Services::load(file) {
        Ok(r) => r,
        Err(e) => return Err(ConfigParseError::new(path.as_ref(), e).into()),
    };

    Ok(config)
}
