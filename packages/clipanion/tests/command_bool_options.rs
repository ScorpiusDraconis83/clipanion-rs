use clipanion::{prelude::*, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    #[cli::option("-D,--development", default = false)]
    development: bool,

    #[cli::option("-P,--production", default = false)]
    production: bool,

    #[cli::option("-T,--test", default = false)]
    test: bool,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["-DP"], |command| {
    assert_eq!(command.development, true);
    assert_eq!(command.production, true);
    assert_eq!(command.test, false);
});

test_cli_success!(it_works_with_no_args, MyCli, MyCommand, &["-PTD"], |command| {
    assert_eq!(command.development, true);
    assert_eq!(command.production, true);
    assert_eq!(command.test, true);
});
