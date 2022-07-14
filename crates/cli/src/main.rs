use clap::ArgMatches;
use context::Context;

mod r#mod;
mod pull;
mod up;

pub(crate) const SUCCESS: u8 = 0;
pub(crate) const INITIALIZATION_FAILED: u8 = 253;
pub(crate) const SERVICE_MANAGER_NOT_RUNNING: u8 = 254;
pub(crate) const SERVICE_MANAGER_RUNNING: u8 = 255;

pub(crate) struct Command {
    pub name: &'static str,
    pub specs: fn(name: &str) -> clap::Command<'static>,
    pub run: fn(context: &Context, args: &ArgMatches) -> u8,
    pub service_manager_state: Option<ServiceManagerState>,
}

pub(crate) enum ServiceManagerState {
    Stopped,
    Running,
}

fn main() {
    std::process::exit(run())
}

fn run() -> i32 {
    // Set up commands.
    let commands = [&r#mod::COMMAND, &pull::COMMAND, &up::COMMAND];

    // Parse arguments.
    let args = parse_command_line(&commands);
    let context = match Context::new(std::env::current_dir().unwrap()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return INITIALIZATION_FAILED.into();
        }
    };

    // Run command.
    process_command_line(&context, &commands, &args).into()
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
                let running = context.runtime().service_manager().port().path().exists();

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

    // This should never happen.
    panic!();
}
