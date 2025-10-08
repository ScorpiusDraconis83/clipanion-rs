use clipanion::{prelude::*, program, test_cli_success};

#[cli::command(default)]
struct MyProxyCommand {
    args: Vec<String>,
}

impl MyProxyCommand {
    fn execute(&self) {
    }
}

#[cli::command(default)]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyProxyCommand(MyProxyCommand),
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &[], |_| {
});
