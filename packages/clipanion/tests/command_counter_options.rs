use clipanion::{prelude::*, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    #[cli::option("-v,--verbose", default = 0, counter)]
    verbose: u8,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_uses_the_default_value, MyCli, MyCommand, &[], |command| {
    assert_eq!(command.verbose, 0);
});

test_cli_success!(it_increments_the_value_on_first_use, MyCli, MyCommand, &["-v"], |command| {
    assert_eq!(command.verbose, 1);
});

test_cli_success!(it_increments_the_value_on_each_use, MyCli, MyCommand, &["-v", "-v", "-v"], |command| {
    assert_eq!(command.verbose, 3);
});

test_cli_success!(it_supports_batched_options, MyCli, MyCommand, &["-vvvv"], |command| {
    assert_eq!(command.verbose, 4);
});
