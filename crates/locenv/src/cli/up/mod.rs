use super::{Command, ServiceManagerState};
use crate::service_manager::requests::Request;
use crate::SUCCESS;
use context::Context;
use dirtree::File;
use service::{ApplicationConfiguration, PlatformConfigurations, ServiceDefinition};
use std::borrow::Cow;
use std::collections::HashMap;
use std::env::current_exe;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::process::{Child, Stdio};
use std::time::SystemTime;

pub(super) const COMMAND: Command = Command {
    name: "up",
    specs: |name| clap::Command::new(name).about("Start all services"),
    run,
    service_manager_state: Some(ServiceManagerState::Stopped),
};

pub const OPEN_CONFIGURATION_FAILED: u8 = 1;
pub const READ_CONFIGURATION_FAILED: u8 = 2;
pub const INVALID_REPOSITORY_OPTION: u8 = 3;
pub const GIT_CLONE_FAILED: u8 = 4;
pub const GIT_OPEN_FAILED: u8 = 5;
pub const GIT_PULL_FAILED: u8 = 6;
pub const OPEN_DEFINITION_FAILED: u8 = 50;
pub const READ_DEFINITION_FAILED: u8 = 51;
pub const PLATFORM_NOT_SUPPORTED: u8 = 52;
pub const DUPLICATED_CONFIGURATION: u8 = 53;
pub const BUILD_FAILED: u8 = 54;
pub const GET_SERVICE_MANAGER_STATUS_FAILED: u8 = 55;

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

    // Download and build repositories.
    let mut services: HashMap<&str, PlatformConfigurations> = HashMap::new();

    for (name, config) in &config.configurations {
        let repo = context
            .project()
            .runtime(false)
            .unwrap()
            .configurations(false)
            .unwrap()
            .by_name(Cow::Borrowed(name.as_str()));
        let path = repo.path();
        let service_definition = repo.service_definition();
        let state = repo.build_state(false).unwrap();

        // Download.
        let build: bool = if !path.exists() {
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

            true
        } else if !state.built_time().path().exists() {
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
            }

            true
        } else {
            false
        };

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

        let (config, platform) = match service.flatten() {
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
        if build {
            if let Some(script) = &config.build {
                let mut engine = script::Engine::new(context, &path);

                println!("Building {}...", name);

                if let Err(e) = engine.run(&script, Some(&platform)) {
                    let msg = match e {
                        script::RunError::LoadError(m) => m,
                        script::RunError::ArgumentError(e) => {
                            panic!("Cannot convert script argument to Lua value: {}", e)
                        }
                        script::RunError::ExecError(m) => m,
                    };

                    eprintln!("{}", msg);
                    return BUILD_FAILED;
                }
            }

            state.built_time().write(&SystemTime::now()).unwrap();
        }

        if services.insert(&name, config).is_some() {
            eprintln!("Duplicated configuration '{}'", name);
            return DUPLICATED_CONFIGURATION;
        }
    }

    // Start Service Manager.
    if let Some(exit) = start_service_manager() {
        return exit;
    }

    SUCCESS
}

fn start_service_manager() -> Option<u8> {
    // Launch Service Manager.
    let process = std::process::Command::new(current_exe().unwrap())
        .env("LOCENV_PROCESS_MODE", "service-manager")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut guard = LaunchGuard {
        process,
        success: false,
    };

    let mut stdin = guard.stdin.take().unwrap();
    let mut stdout = guard.stdout.take().unwrap();

    // Wait for initialization completed.
    let mut data: Vec<u8> = Vec::with_capacity(256);

    write!(data, "{} HTTP/1.1\r\n\r\n", Request::GetStatus).unwrap();

    if let Err(e) = stdin.write_all(&data) {
        eprintln!(
            "Failed to send a request to get Service Manager status: {}",
            e
        );
        return Some(GET_SERVICE_MANAGER_STATUS_FAILED);
    } else if let Err(e) = stdin.flush() {
        eprintln!(
            "Failed to send a request to get Service Manager status: {}",
            e
        );
        return Some(GET_SERVICE_MANAGER_STATUS_FAILED);
    }

    data.clear();

    'read_output: loop {
        // Read output.
        let mut buffer = [0u8; 256];
        let count = match stdout.read(&mut buffer) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to read Service Manager status: {}", e);
                return Some(GET_SERVICE_MANAGER_STATUS_FAILED);
            }
        };

        if count == 0 {
            eprintln!("End of file has been reached while reading Service Manager status");
            return Some(GET_SERVICE_MANAGER_STATUS_FAILED);
        }

        data.extend_from_slice(&buffer);

        // Check status code.
        for i in 0..data.len() {
            let remaining = &data[i..data.len()];

            if remaining.len() < 2 {
                continue 'read_output;
            } else if remaining[0] != b'\r' {
                continue;
            } else if remaining[1] != b'\n' {
                eprintln!("Got an unexpected response from Service Manager");
                return Some(GET_SERVICE_MANAGER_STATUS_FAILED);
            }

            let line = match std::str::from_utf8(&data[0..i]) {
                Ok(r) => r,
                Err(_) => {
                    eprintln!("Got an unexpected response from Service Manager");
                    return Some(GET_SERVICE_MANAGER_STATUS_FAILED);
                }
            };

            let components: Vec<&str> = line.splitn(3, ' ').collect();

            if components.len() < 2 || components[1] != "200" {
                eprintln!("Got an unexpected response from Service Manager");
                return Some(GET_SERVICE_MANAGER_STATUS_FAILED);
            }

            break 'read_output;
        }
    }

    // Launch successfully.
    guard.success = true;

    None
}

struct LaunchGuard {
    process: Child,
    success: bool,
}

impl Deref for LaunchGuard {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        &self.process
    }
}

impl DerefMut for LaunchGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.process
    }
}

impl Drop for LaunchGuard {
    fn drop(&mut self) {
        if self.success {
            return;
        }

        match self.process.kill() {
            Ok(_) => {
                self.process.wait().unwrap();
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::InvalidInput => {}
                _ => panic!("Failed to kill process {}", self.process.id()),
            },
        };
    }
}
