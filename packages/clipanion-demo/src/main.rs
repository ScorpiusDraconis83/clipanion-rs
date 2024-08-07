use std::process::ExitCode;

use clipanion::{advanced::{Cli, Info}, cli};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Oh no! Something bad happened!")]
    ArbitraryError,
}

#[derive(Debug)]
#[cli::command]
#[cli::path("cp")]
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

#[derive(Debug)]
#[cli::command]
#[cli::path("yarn")]
#[cli::path("yarn", "install")]
struct YarnInstall {
}

impl YarnInstall {
    pub fn execute(&self) {
        println!("{:?}", self);
    }
}

#[derive(Debug)]
#[cli::command(proxy)]
#[cli::path("yarn", "run")]
struct YarnRun {
    script: String,
    args: Vec<String>,
}

impl YarnRun {
    pub fn execute(&self) {
        println!("{:?}", self);
    }
}

#[cli::command(proxy)]
#[cli::path("yarn")]
struct YarnRunDefault {
    script: String,
    args: Vec<String>,
}

impl YarnRunDefault {
    pub fn execute(&self) -> ExitCode {
        let mut argv = vec!["yarn".to_string(), "run".to_string(), self.script.clone()];
        argv.extend(self.args.clone());

        MyCli::run(self.cli_info.with_argv(argv))
    }
}

clipanion::program!(MyCli, [
    Cp,
    Unimplemented,
    YarnInstall,
    YarnRun,
    YarnRunDefault,
]);

fn main() -> ExitCode {
    MyCli::run_default()
}
