use clipanion::{prelude::*, test_cli_failure, test_cli_success, Error};

#[cli::command(default)]
struct MyProxyCommand {
    args: Vec<String>,
}

impl MyProxyCommand {
    fn execute(&self) {
    }
}

#[cli::command]
#[cli::path("foo")]
struct MyCommand {
    arg: String,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyProxyCommand(MyProxyCommand),
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["foo", "bar"], |command| {
    assert_eq!(command.arg, "bar");
});

test_cli_failure!(it_reports_an_error_when_the_commands_are_wrong, MyCli, &["foo"], |error| {
    assert_eq!(error, Error::NotFound(vec![MyCommand::command_spec().unwrap()]));
});
