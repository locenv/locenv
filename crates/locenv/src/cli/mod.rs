use clap::ArgMatches;
use context::Context;
use std::unreachable;

mod module;
mod pull;
mod up;

struct Command {
    name: &'static str,
    specs: fn(name: &str) -> clap::Command<'static>,
    run: fn(context: &Context, args: &ArgMatches) -> u8,
    service_manager_state: Option<ServiceManagerState>,
}

enum ServiceManagerState {
    Stopped,
    Running,
}

pub const SERVICE_MANAGER_NOT_RUNNING: u8 = 252;
pub const SERVICE_MANAGER_RUNNING: u8 = 253;
pub const INITIALIZATION_FAILED: u8 = 254;

pub fn run() -> u8 {
    // Set up commands.
    let commands = [
        &self::module::COMMAND,
        &self::pull::COMMAND,
        &self::up::COMMAND,
    ];

    // Parse arguments.
    let args = parse_command_line(&commands);
    let context = match Context::new(std::env::current_dir().unwrap()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return INITIALIZATION_FAILED;
        }
    };

    // Run command.
    process_command_line(&context, &commands, &args)
}

fn parse_command_line(commands: &[&Command]) -> ArgMatches {
    let mut args = clap::command!().subcommand_required(true);

    for command in commands {
        args = args.subcommand((command.specs)(command.name));
    }

    args.get_matches()
}

fn process_command_line(context: &Context, commands: &[&Command], args: &ArgMatches) -> u8 {
    for command in commands {
        if let Some(args) = args.subcommand_matches(command.name) {
            // Check service manager state.
            if let Some(state) = &command.service_manager_state {
                let running = context
                    .project()
                    .runtime()
                    .service_manager()
                    .port()
                    .path()
                    .exists();

                match state {
                    ServiceManagerState::Stopped => {
                        if running {
                            eprintln!("The Service Manager is currently running, please stop it with 'stop' before running this command");
                            return SERVICE_MANAGER_RUNNING;
                        }
                    }
                    ServiceManagerState::Running => {
                        if !running {
                            eprintln!("The Service Manager is not running, please start it with 'up' command before running this command");
                            return SERVICE_MANAGER_NOT_RUNNING;
                        }
                    }
                }
            }

            return (command.run)(context, args);
        }
    }

    unreachable!();
}
