use crate::command::Command;
use config::Services;
use context::Context;
use std::error::Error;

pub fn command<'run>() -> Command<'run> {
    Command {
        name: "pull",
        specs: |name| clap::Command::new(name).about("Update all services"),
        manager_running: Some(false),
        run: |ctx, args| Box::pin(run(ctx, args)),
    }
}

async fn run(context: &Context, _: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    // Load config.
    let conf = Services::from_file(context.project().services_config())?;

    // Update local repositories.
    for (n, s) in &conf {
        let path = context.runtime().repositories().by_name(n).path();

        if path.is_dir() {
            repository::update(&path, &s.repository).await?;
        } else {
            repository::download(&path, &s.repository).await?;
        }
    }

    Ok(())
}
