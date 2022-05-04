use clap::{command, Command};

mod up;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments.
    let args = command!()
        .subcommand(
            Command::new("up").about("Start all services"),
        )
        .subcommand_required(true)
        .get_matches();

    // Handle sub-commands.
    if let Some(cmd) = args.subcommand_matches("up") {
        up::run().await?
    }

    Ok(())
}
