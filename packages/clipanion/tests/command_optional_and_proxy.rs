use clipanion::{prelude::*, program, test_cli_success};

#[cli::command(default, proxy)]
struct MyCommand {
    value: Option<String>,
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

test_cli_success!(it_works_with_the_positional, MyCli, MyCommand, &["foo", "--version"], |command| {
    assert_eq!(command.value, Some("foo".to_string()));
    assert_eq!(command.rest, vec!["--version"]);
});

test_cli_success!(it_works_without_the_positional, MyCli, MyCommand, &["--version"], |command| {
    assert_eq!(command.value, None);
    assert_eq!(command.rest, vec!["--version"]);
});
