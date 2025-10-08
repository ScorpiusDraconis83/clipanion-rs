use clipanion::{prelude::*, program, test_cli_success};

#[cli::command]
#[cli::path("foo")]
#[cli::path("bar")]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works_with_foo, MyCli, MyCommand, &["foo"], |_| {
});

test_cli_success!(it_works_with_bar, MyCli, MyCommand, &["bar"], |_| {
});
