use clipanion::{advanced::Environment, details::{CommandController, CommandProvider}, prelude::*, program, test_cli_failure, test_cli_success};
use clipanion_core::Error;

#[cli::command]
#[cli::path("foo", "bar")]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_works, MyCli, MyCommand, &["foo", "bar"], |_| {
});

test_cli_failure!(it_requires_the_full_path, MyCli, MyCommand, &["foo"], |error| {
    assert_eq!(error, Error::NotFound(vec![]));
});

test_cli_failure!(it_checks_the_path_segments, MyCli, MyCommand, &["foo", "baz"], |error| {
    assert_eq!(error, Error::NotFound(vec![]));
});
