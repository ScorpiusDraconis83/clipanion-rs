use clipanion::{prelude::*, program, test_cli_failure, test_cli_success, Error};

#[cli::command]
#[cli::path("foo", "bar")]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["foo", "bar"], |_| {
});

test_cli_failure!(it_requires_the_full_path, MyCli, &["foo"], |error| {
    assert_eq!(error, Error::NotFound(vec![MyCommand::command_spec().unwrap()]));
});

test_cli_failure!(it_checks_the_path_segments, MyCli, &["foo", "baz"], |error| {
    assert_eq!(error, Error::NotFound(vec![MyCommand::command_spec().unwrap()]));
});
