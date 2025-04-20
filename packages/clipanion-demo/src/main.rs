use std::{process::ExitCode, str::FromStr};

use clipanion::prelude::*;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Oh no! Something bad happened!")]
    ArbitraryError,
}

#[derive(Debug)]
struct HexColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl FromStr for HexColor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 7 || !s.starts_with('#') {
            return Err(Error::ArbitraryError);
        }

        let r = u8::from_str_radix(&s[1..3], 16).map_err(|_| Error::ArbitraryError)?;
        let g = u8::from_str_radix(&s[3..5], 16).map_err(|_| Error::ArbitraryError)?;
        let b = u8::from_str_radix(&s[5..7], 16).map_err(|_| Error::ArbitraryError)?;

        Ok(HexColor { r, g, b })
    }
}

#[derive(Debug)]
#[cli::command]
#[cli::path("cp")]
struct Cp {
    #[cli::option("-r,--recursive", help = "Copy directories recursively", default = false)]
    recursive: bool,

    sources: Vec<String>,
    destination: String,
}

impl Cp {
    pub fn execute(&self) {
        println!("{:?}", self);
    }
}

#[derive(Debug)]
#[cli::command]
#[cli::path("grep")]
struct Grep {
    #[cli::option("--color")]
    color: Option<HexColor>,
}

impl Grep {
    pub fn execute(&self) {
        println!("{:?}", self);
    }
}

#[derive(Debug)]
#[cli::command]
#[cli::path("ssh")]
struct Ssh {
    #[cli::option("-p,--port", help = "Port to connect to", default = 22)]
    port: u16,

    #[cli::option("--user", help = "User to connect as", default = "root".to_string())]
    user: String,

    host: String,
}

impl Ssh {
    pub fn execute(&self) {
        println!("{:?}", self);
    }
}

#[cli::command]
#[cli::path("unimplemented")]
struct Unimplemented {}

impl Unimplemented {
    pub fn execute(&self) -> anyhow::Result<()> {
        Err(Error::ArbitraryError)?
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

#[cli::command(proxy)]
#[cli::path("colorlist")]
struct ColorList {
    #[cli::option("--color", help = "Color to list", default = vec![])]
    color_options: Vec<HexColor>,

    colors: Vec<HexColor>,
}

impl ColorList {
    pub fn execute(&self) {
        println!("{:?}", self.colors);
    }
}

clipanion::program!(MyCli, [
    ColorList,
    Cp,
    Grep,
    Ssh,
    Unimplemented,
    YarnInstall,
    YarnRun,
    YarnRunDefault,
]);

#[tokio::main()]
async fn main() -> ExitCode {
    MyCli::run_default()
}

#[test]
fn it_should_support_program() {
    #[cli::command(default)]
    struct MyCommandSync {}

    impl MyCommandSync {
        fn execute(&self) -> () {
        }
    }

    clipanion::program!(MyCliSync, [
        MyCommandSync,
    ]);

    MyCliSync::run_default();
}

#[tokio::test]
async fn it_should_support_program_async() {
    #[cli::command(default)]
    struct MyCommandAsync {}

    impl MyCommandAsync {
        async fn execute(&self) -> () {
        }
    }

    clipanion::program_async!(MyCliAsync, [
        MyCommandAsync,
    ]);

    MyCliAsync::run_default().await;
}
