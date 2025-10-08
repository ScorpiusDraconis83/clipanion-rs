use clipanion::{prelude::*, program, test_cli_success};

#[cli::command]
#[cli::path("foo")]
struct MyCommand {
    #[cli::positional(is_prefix = true)]
    prefix: String,

    #[cli::positional]
    positional: String,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["hello", "foo", "world"], |command| {
    assert_eq!(command.prefix, "hello");
    assert_eq!(command.positional, "world");
});
