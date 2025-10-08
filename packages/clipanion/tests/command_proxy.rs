use clipanion::{prelude::*, test_cli_success};

#[cli::command(proxy)]
#[cli::path("foo")]
struct MyCommand {
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

test_cli_success!(it_works, MyCli, MyCommand, &["foo", "-h"], |command| {
    assert_eq!(command.rest, vec!["-h"]);
});
