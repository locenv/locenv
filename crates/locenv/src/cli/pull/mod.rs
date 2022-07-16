use super::{Command, ServiceManagerState};
use crate::SUCCESS;
use context::Context;
use service::{ApplicationConfiguration, ServiceDefinition};
use std::borrow::Cow;
use std::time::SystemTime;

pub(super) const COMMAND: Command = Command {
    name: "pull",
    specs: |name| clap::Command::new(name).about("Update all services"),
    run,
    service_manager_state: Some(ServiceManagerState::Stopped),
};

pub const OPEN_CONFIGURATION_FAILED: u8 = 1;
pub const READ_CONFIGURATION_FAILED: u8 = 2;
pub const INVALID_REPOSITORY_OPTION: u8 = 3;
pub const GIT_OPEN_FAILED: u8 = 4;
pub const GIT_PULL_FAILED: u8 = 5;
pub const GIT_CLONE_FAILED: u8 = 6;
pub const OPEN_DEFINITION_FAILED: u8 = 50;
pub const READ_DEFINITION_FAILED: u8 = 51;
pub const PLATFORM_NOT_SUPPORTED: u8 = 52;
pub const BUILD_FAILED: u8 = 53;

fn run(context: &Context, _: &clap::ArgMatches) -> u8 {
    // Load config.
    let path = context.project().services();
    let config: ApplicationConfiguration = match yaml::load_file(&path) {
        Ok(r) => r,
        Err(e) => {
            return match e {
                yaml::FileError::OpenFailed(e) => {
                    eprintln!("Failed to open {}: {}", path.display(), e);
                    OPEN_CONFIGURATION_FAILED
                }
                yaml::FileError::ParseFailed(e) => {
                    eprintln!("Failed to read {}: {}", path.display(), e);
                    READ_CONFIGURATION_FAILED
                }
            }
        }
    };

    // Update and rebuild repositories.
    for (name, config) in &config.configurations {
        let repo = context
            .runtime()
            .configurations()
            .by_name(Cow::Borrowed(&name));
        let path = repo.path();
        let service_definition = repo.service_definition();
        let state = repo.build_state();

        // Update.
        if path.exists() {
            println!("Updating {}...", name);

            if let Err(e) = service::repository::update(&config.repository, &path) {
                return match e {
                    service::repository::UpdateError::InvalidOption(name) => {
                        eprintln!("Invalid value for repository option '{}'", name);
                        INVALID_REPOSITORY_OPTION
                    }
                    service::repository::UpdateError::GitOpenFailed(e) => {
                        eprintln!(
                            "Failed to open {} as a Git repository: {}",
                            path.display(),
                            e
                        );
                        GIT_OPEN_FAILED
                    }
                    service::repository::UpdateError::GitFindOriginFailed(e) => {
                        eprintln!(
                            "Failed to find 'origin' remote on repository {}: {}",
                            path.display(),
                            e
                        );
                        GIT_PULL_FAILED
                    }
                    service::repository::UpdateError::GitFetchOriginFailed(e) => {
                        eprintln!("Failed to pull {}: {}", path.display(), e);
                        GIT_PULL_FAILED
                    }
                };
            } else if state.built_time().path().exists() {
                continue;
            }
        } else {
            println!("Downloading {} to {}...", name, path.display());

            if let Err(e) = service::repository::download(&config.repository, &path) {
                return match e {
                    service::repository::DownloadError::InvalidOption(name) => {
                        eprintln!("Invalid value for repository option '{}'", name);
                        INVALID_REPOSITORY_OPTION
                    }
                    service::repository::DownloadError::GitCloneFailed(e) => {
                        eprintln!("Failed to clone the repository: {}", e);
                        GIT_CLONE_FAILED
                    }
                };
            }
        }

        // Read service definition.
        let service: ServiceDefinition = match yaml::load_file(&service_definition) {
            Ok(r) => r,
            Err(e) => {
                return match e {
                    yaml::FileError::OpenFailed(e) => {
                        eprintln!("Failed to open {}: {}", service_definition.display(), e);
                        OPEN_DEFINITION_FAILED
                    }
                    yaml::FileError::ParseFailed(e) => {
                        eprintln!("Failed to read {}: {}", service_definition.display(), e);
                        READ_DEFINITION_FAILED
                    }
                }
            }
        };

        let service = match service.flatten() {
            Some(v) => v,
            None => {
                eprintln!(
                    "The repository for configuration '{}' does not support this platform",
                    name
                );
                return PLATFORM_NOT_SUPPORTED;
            }
        };

        // Build.
        if let Some(script) = &service.build {
            let mut engine = script::Engine::new(context, &path);

            println!("Building {}", name);

            if let Err(e) = engine.run(&script) {
                let msg = match e {
                    script::RunError::LoadError(m) => m,
                    script::RunError::ExecError(m) => m,
                };

                eprintln!("{}", msg);
                return BUILD_FAILED;
            }
        }

        state.built_time().write(&SystemTime::now()).unwrap();
    }

    SUCCESS
}
