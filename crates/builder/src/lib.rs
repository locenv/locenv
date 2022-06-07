use config::service::BuildConfigurations;
use context::Context;
use module::Module;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum BuildError {
    MissingModule(String),
}

pub fn build(context: &Context, conf: &BuildConfigurations) -> Result<(), BuildError> {
    for step in &conf.steps {
        let module = match Module::find(context, &step.uses) {
            Some(r) => r,
            None => return Err(BuildError::MissingModule(step.uses.clone())),
        };
    }

    Ok(())
}

// BuildError

impl Error for BuildError {}

impl Display for BuildError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::MissingModule(name) => {
                write!(f, "This system does not have module '{}' installed", name)
            }
        }
    }
}
