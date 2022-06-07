use self::command::Command;
use self::errors::IncorrectManagerState;
use context::Context;
use std::error::Error;

mod command;
mod errors;
mod pull;
mod up;

fn main() {
    std::process::exit(run())
}

fn run() -> i32 {
    // Set up commands.
    let commands = [pull::command(), up::command()];

    // Parse arguments.
    let args = parse_command_line(&commands);
    let context = match Context::new(std::env::current_dir().unwrap()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    // Run command.
    if let Err(e) = process_command(&context, &commands, &args) {
        eprintln!("{}", e);
        return 1;
    }

    0
}

fn parse_command_line(commands: &[Command]) -> clap::ArgMatches {
    let mut args = clap::command!().subcommand_required(true);

    for command in commands {
        args = args.subcommand((command.specs)(command.name));
    }

    args.get_matches()
}

fn process_command(
    context: &Context,
    commands: &[Command],
    args: &clap::ArgMatches,
) -> Result<(), Box<dyn Error>> {
    for command in commands {
        if let Some(cmd) = args.subcommand_matches(command.name) {
            if let Some(v) = command.manager_running {
                if manager::is_running(context) != v {
                    return Err(IncorrectManagerState::new(v).into());
                }
            }

            return (command.run)(context, cmd);
        }
    }

    // This should never happen.
    panic!();
}
