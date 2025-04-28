use clipanion::{prelude::*, program, test_cli_success, Environment};

#[cli::command]
#[cli::path("foo")]
#[cli::path("bar")]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_works_with_foo, MyCli, MyCommand, &["foo"], |_| {
});

test_cli_success!(it_works_with_bar, MyCli, MyCommand, &["bar"], |_| {
});
