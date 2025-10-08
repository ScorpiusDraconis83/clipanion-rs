use clipanion::{prelude::*, program, test_cli_success};

#[cli::command(default, proxy)]
struct MyCommand {
    value: String,
    rest: Vec<String>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["foo", "--test", "bar", "baz"], |command| {
    assert_eq!(command.value, "foo");
    assert_eq!(command.rest, vec!["--test", "bar", "baz"]);
});
