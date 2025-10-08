use clipanion::{prelude::*, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    #[cli::option("--my-option", default = false)]
    value: bool,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["--my-option"], |command| {
    assert_eq!(command.value, true);
});

test_cli_success!(it_works_with_no_args, MyCli, MyCommand, &[], |command| {
    assert_eq!(command.value, false);
});
