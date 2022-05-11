use crate::command::Command;
use crate::context::Context;
use clap::ArgMatches;
use config::Services;
use std::{fs::File, path::Path};

mod config;
mod repository;

pub fn command<'args>() -> Command<'args> {
    Command {
        name: "up",
        specs: |name| clap::Command::new(name).about("Start all services"),
        run: |context, args| Box::pin(run(context, args)),
    }
}

async fn run(ctx: &Context, _: &ArgMatches) -> Result<(), String> {
    // Load config.
    let conf = load_config(ctx.path.join("locenv-services.yml"))?;

    // Check if services already running.
    if is_running(ctx)? {
        return Err(String::from("The services already running"));
    }

    // Update local repositories.
    for (n, s) in &conf {
        if let Err(e) = repository::update(&s.repository).await {
            return Err(format!("Failed to update local repository for {}: {}", n, e));
        }
    }

    Ok(())
}

fn is_running(ctx: &Context) -> Result<bool, String> {
    let mut port = ctx.path.join(".locenv");

    port.push("service-manager");
    port.push("port");

    match std::fs::metadata(&port) {
        Ok(r) => if r.is_file() {
            Ok(true)
        } else {
            Err(format!("Unexpected file type for {}", port.display()))
        },
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Ok(false),
            _ => Err(format!("Failed to get metadata of {}: {}", port.display(), e))
        }
    }
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
