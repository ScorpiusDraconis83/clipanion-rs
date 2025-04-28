use clipanion::{prelude::*, program, test_cli_success};

#[cli::command]
#[cli::path("foo")]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_works, MyCli, MyCommand, &["foo"], |_| {
});
