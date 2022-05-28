use crate::command::Command;
use config::Services;
use context::Context;
use std::error::Error;

pub fn command<'run>() -> Command<'run> {
    Command {
        name: "up",
        specs: |name| clap::Command::new(name).about("Start all services"),
        manager_running: Some(false),
        run: |ctx, args| Box::pin(run(ctx, args)),
    }
}

async fn run(ctx: &Context, _: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    // Load config.
    let conf = Services::from_file(ctx.services_config())?;

    // Create local repositories.
    for (name, service) in &conf {
        // Build a path to repository.
        let path = ctx.repository_dir(name);

        if path.is_dir() {
            continue;
        }

        // Download.
        println!("Downloading {} to {}", name, path.display());

        repository::download(&path, &service.repository).await?;
    }

    Ok(())
}
