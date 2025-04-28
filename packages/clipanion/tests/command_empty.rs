use clipanion::{prelude::*, program, test_cli_success};

#[cli::command(default)]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_works, MyCli, MyCommand, &[], |_| {
});
