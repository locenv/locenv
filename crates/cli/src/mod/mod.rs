use crate::{Command, ServiceManagerState, SUCCESS};
use clap::{value_parser, Arg};
use context::Context;
use module::{InstallError, Module, PackageId, UpdateError};
use std::borrow::Cow;

pub(crate) const COMMAND: Command = Command {
    name: "mod",
    specs,
    run,
    service_manager_state: Some(ServiceManagerState::Stopped),
};

pub const INVALID_IDENTIFIER: u8 = 1;
pub const GET_PACKAGE_FAILED: u8 = 2;
pub const ALREADY_INSTALLED: u8 = 3;
pub const NOT_INSTALLED: u8 = 4;

fn specs(name: &str) -> clap::Command<'static> {
    let install = clap::Command::new("install").about("Install a module").arg(
        Arg::new("id")
            .help("Package identifier (e.g. github:locenv/mod-autoconf)")
            .required(true)
            .value_parser(value_parser!(PackageId)),
    );
    let update = clap::Command::new("update")
        .about("Update installed module")
        .arg(
            Arg::new("name")
                .help("Name of the module to update")
                .required(true),
        );

    clap::Command::new(name)
        .about("Manage the modules")
        .subcommand_required(true)
        .subcommand(install)
        .subcommand(update)
}

fn run(context: &Context, args: &clap::ArgMatches) -> u8 {
    if let Some(args) = args.subcommand_matches("install") {
        let id: &PackageId = args.get_one("id").unwrap();

        if let Err(e) = Module::install(context, id) {
            match e {
                InstallError::InvalidIdentifier => {
                    eprintln!(
                        "'{}' is not a valid identifer for '{}'",
                        id.name(),
                        id.registry()
                    );
                    INVALID_IDENTIFIER
                }
                InstallError::GetPackageFailed(e) => {
                    eprintln!("Failed to get a package to install: {}", e);
                    GET_PACKAGE_FAILED
                }
                InstallError::AlreadyInstalled(name) => {
                    eprintln!("The module '{}' is already installed", name);
                    ALREADY_INSTALLED
                }
            }
        } else {
            SUCCESS
        }
    } else if let Some(args) = args.subcommand_matches("update") {
        let name = args.get_one::<String>("name").unwrap();

        if let Err(e) = Module::update(context, Cow::Borrowed(&name)) {
            match e {
                UpdateError::NotInstalled => {
                    eprintln!("The specified module is not installed");
                    NOT_INSTALLED
                }
                UpdateError::GetPackageFailed(e) => {
                    eprintln!("Failed to get a package to update: {}", e);
                    GET_PACKAGE_FAILED
                }
                UpdateError::AlreadyLatest => {
                    println!("The specified module is already in latest version");
                    SUCCESS
                }
            }
        } else {
            SUCCESS
        }
    } else {
        panic!("Sub-command not implemented")
    }
}
