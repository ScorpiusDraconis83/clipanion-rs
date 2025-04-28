use clipanion::{advanced::Environment, details::CommandProvider, prelude::*, program, test_cli_failure, test_cli_success};
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
    let command_error = match error {
        Error::CommandError(_, command_error) => command_error,
        _ => panic!("expected command error"),
    };

    assert_eq!(command_error, CommandError::MissingOptionArguments("--my-option".to_string()));
});
