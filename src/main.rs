use clap::{command, Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments.
    let args = command!()
        .subcommand(
            Command::new("up").about("Start all services"),
        )
        .get_matches();

    // Handle sub-commands.
    if let Some(up) = args.subcommand_matches("up") {
    }

    println!("Hello, world!");
    Ok(())
}
