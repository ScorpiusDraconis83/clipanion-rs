use clipanion::{prelude::*, program, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    value: Option<String>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["foo"], |command| {
    assert_eq!(command.value, Some("foo".to_string()));
});
