use clipanion::{prelude::*, program, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    rest: Vec<String>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_works_at_the_start_of_the_command_line, MyCli, MyCommand, &["--", "foo", "bar", "--test"], |command| {
    assert_eq!(command.rest, vec!["foo", "bar", "--test"]);
});

test_cli_success!(it_works_at_the_end_of_the_command_line, MyCli, MyCommand, &["foo", "bar", "--", "--test"], |command| {
    assert_eq!(command.rest, vec!["foo", "bar", "--test"]);
});


