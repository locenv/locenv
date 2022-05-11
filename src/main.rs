mod command;
mod scp;
mod up;

fn main() {
    std::process::exit(run())
}

fn run() -> i32 {
    // Set up commands.
    let commands = [
        up::command()
    ];

    // Parse arguments.
    let args = parse_command_line(&commands);

    // Set up Tokio.
    let tokio = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to setup runtime: {}", e);
            return 1
        }
    };

    // Run command.
    match tokio.block_on(async { process_command(&commands, &args).await }) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    }
}

fn parse_command_line(commands: &[command::Command]) -> clap::ArgMatches {
    let mut args = clap::command!().subcommand_required(true);

    for command in commands {
        args = args.subcommand((command.specs)(command.name));
    }

    args.get_matches()
}

async fn process_command<'args>(commands: &[command::Command<'args>], args: &'args clap::ArgMatches) -> Result<(), String> {
    for command in commands {
        if let Some(cmd) = args.subcommand_matches(command.name) {
            (command.run)(cmd).await?;
            return Ok(());
        }
    }

    // This should never happen.
    panic!()
}
