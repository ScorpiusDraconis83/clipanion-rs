use std::process::ExitCode;

use clipanion::cli;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Oh no! Something bad happened!")]
    ArbitraryError,
}

#[cli::command]
#[cli::path("cp")]
#[derive(Debug)]
struct Cp {
    #[cli::option("-r,--recursive", help = "Copy directories recursively", initial = false)]
    recursive: bool,

    sources: Vec<String>,
    destination: String,
}

impl Cp {
    pub fn execute(&self) {
        println!("{:?}", self);
    }
}

#[cli::command]
#[cli::path("unimplemented")]
struct Unimplemented {}

impl Unimplemented {
    pub fn execute(&self) -> Result<(), Error> {
        Err(Error::ArbitraryError)
    }
}

fn main() -> ExitCode {
    clipanion::new![Cp, Unimplemented].run_default()
}
