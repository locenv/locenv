use std::env::VarError;

mod cli;

pub const SUCCESS: u8 = 0;
pub const INVALID_LOCENV_PROCESS_MODE: u8 = 255;

fn main() {
    std::process::exit(run() as _)
}

fn run() -> u8 {
    let var = "LOCENV_PROCESS_MODE";

    match std::env::var(var) {
        Ok(_) => todo!(),
        Err(e) => match e {
            VarError::NotPresent => cli::run(),
            VarError::NotUnicode(_) => {
                eprintln!("{} has invalid value", var);
                INVALID_LOCENV_PROCESS_MODE
            },
        },
    }
}
