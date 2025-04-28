use clipanion::{advanced::Environment, details::{CommandController, CommandProvider}, prelude::*, program, test_cli_failure, test_cli_success};
use clipanion_core::{CommandError, Error};

#[cli::command(default)]
struct MyCommand {
    #[cli::option("--my-option")]
    value: bool,
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_works, MyCli, MyCommand, &["--my-option"], |command| {
    assert_eq!(command.value, true);
});

test_cli_failure!(it_requires_the_option_to_be_present, MyCli, MyCommand, &[], |error| {
    assert_eq!(error, Error::CommandError(&MyCommand::command_spec().unwrap(), CommandError::MissingOptionArguments("--my-option".to_string())));
});
