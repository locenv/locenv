use crate::command::Command;
use crate::context::Context;
use std::error::Error;

pub fn command<'args>() -> Command<'args> {
    Command {
        name: "pull",
        specs: |name| clap::Command::new(name).about("Update all services"),
        run: |ctx, args| Box::pin(run(ctx, args)),
    }
}

async fn run(_: &Context, _: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    Ok(())
}
