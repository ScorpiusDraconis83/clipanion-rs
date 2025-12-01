use clipanion::{prelude::*, test_cli_success};

#[cli::command(default, proxy)]
struct MyCommand {
    #[cli::option("--arg")]
    arg: Vec<String>,

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

test_cli_success!(it_works, MyCli, MyCommand, &["--arg", "foo", "bar"], |command| {
    assert_eq!(command.arg, vec!["foo"]);
    assert_eq!(command.rest, vec!["bar"]);
});
