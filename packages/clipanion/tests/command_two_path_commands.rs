use clipanion::{prelude::*, test_cli_failure, test_cli_success, Error};

#[cli::command]
#[cli::path("foo")]
struct MyFooCommand {
}

impl MyFooCommand {
    fn execute(&self) {
    }
}

#[cli::command]
#[cli::path("bar")]
struct MyBarCommand {
}

impl MyBarCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyFooCommand(MyFooCommand),
    MyBarCommand(MyBarCommand),
}

test_cli_success!(it_works_with_foo, MyCli, MyFooCommand, &["foo"], |_| {
});

test_cli_success!(it_works_with_bar, MyCli, MyBarCommand, &["bar"], |_| {
});

test_cli_failure!(it_reports_an_error_when_the_commands_are_wrong, MyCli, &[], |error| {
    assert_eq!(error, Error::NotFound(vec![MyFooCommand::command_spec().unwrap(), MyBarCommand::command_spec().unwrap()]));
});

test_cli_failure!(it_reports_the_wrong_command_and_only_this_one, MyCli, &["foo", "extraneous"], |error| {
    assert_eq!(error, Error::NotFound(vec![MyFooCommand::command_spec().unwrap()]));
});
