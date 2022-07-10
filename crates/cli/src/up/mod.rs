use crate::command::Command;
use config::service::PlatformConfigurations;
use config::{FromFileError, Services};
use context::Context;
use state::StateManager;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub fn command() -> Command {
    Command {
        name: "up",
        specs: |name| clap::Command::new(name).about("Start all services"),
        manager_running: Some(false),
        run,
    }
}

struct Service<'context, 'config, 'repo, 'state> {
    conf: &'config config::services::Service,
    repo: context::runtime::repositories::repository::Repository<'context, 'repo>,
    state: StateManager<'context, 'state>,
}

#[derive(Debug)]
enum RunError {
    PlatformNotSupported(String),
    ServiceDefinitionOpenError(PathBuf, std::io::Error),
    ServiceDefinitionParseError(PathBuf, Box<dyn Error>),
    BuildError(String, String),
}

fn run(context: &Context, _: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    // Load config.
    let conf = Services::from_file(context.project().services_config())?;

    // Download repositories.
    let mut services: HashMap<&str, Service> = HashMap::new();

    for (name, conf) in &conf {
        let service = Service {
            conf,
            repo: context
                .runtime()
                .repositories()
                .by_name(Cow::Borrowed(name)),
            state: StateManager::new(context.runtime().states().by_name(Cow::Borrowed(name))),
        };

        // Download.
        let path = service.repo.path();

        if !path.is_dir() {
            println!("Downloading {} to {}", name, path.display());
            repository::download(&path, &service.conf.repository)?;
            service.state.clear();
        }

        services.insert(name, service);
    }

    // Build & run.
    for (name, service) in services {
        // Load service definition.
        let def = service.repo.service_definition();
        let conf = match PlatformConfigurations::from_service_definition_file(&def) {
            Ok(r) => match r {
                Some(v) => v,
                None => return Err(RunError::PlatformNotSupported(name.into()).into()),
            },
            Err(e) => match e {
                FromFileError::OpenFailed(e) => {
                    return Err(RunError::ServiceDefinitionOpenError(def, e).into())
                }
                FromFileError::ParseFailed(e) => {
                    return Err(RunError::ServiceDefinitionParseError(def, e.into()).into())
                }
            },
        };

        // Build.
        if service.state.read_built_time().is_none() {
            if let Some(script) = &conf.build {
                let mut engine = script::Engine::new(context);

                println!("Building {}", name);

                if let Err(e) = engine.run(&script) {
                    let msg = match e {
                        script::RunError::LoadError(m) => m,
                        script::RunError::ExecError(m) => m,
                    };

                    return Err(RunError::BuildError(name.into(), msg).into());
                }
            }
        }
    }

    Ok(())
}

// RunError

impl Error for RunError {}

impl Display for RunError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::PlatformNotSupported(s) => write!(f, "Service {} cannot run on this system", s),
            Self::ServiceDefinitionOpenError(p, e) => {
                write!(f, "Failed to open {}: {}", p.display(), e)
            }
            Self::ServiceDefinitionParseError(p, e) => {
                write!(f, "Failed to parse {}: {}", p.display(), e)
            }
            Self::BuildError(_, e) => write!(f, "{}", e),
        }
    }
}
