use clipanion::{prelude::*, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    #[cli::positional]
    positional: String,

    #[cli::positional(is_prefix = true)]
    prefix: String,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["hello", "world"], |command| {
    assert_eq!(command.prefix, "hello");
    assert_eq!(command.positional, "world");
});
