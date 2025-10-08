use clipanion::{prelude::*, test_cli_success};

#[cli::command(default)]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &[], |_| {
});
