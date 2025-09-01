use clipanion::{prelude::*, program, test_cli_failure, test_cli_success, CommandError, Error};

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
    some_value: usize,
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyProxyCommand, MyCommand]);

test_cli_failure!(it_reports_the_proper_error, MyCli, MyCommand, &["foo", "not-a-number"], |error| {
    assert_eq!(error, Error::CommandError(MyCommand::command_spec().unwrap(), CommandError::Custom("invalid digit found in string".to_string())));
});
