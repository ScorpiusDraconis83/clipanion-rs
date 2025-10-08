use clipanion::{prelude::*, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    #[cli::option("--foo")]
    _value: bool,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::command(default)]
struct IndexCommand {
}

impl IndexCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
    IndexCommand(IndexCommand),
}

test_cli_success!(it_works_my_command, MyCli, MyCommand, &["--foo"], |_| {
});

test_cli_success!(it_works_index_command, MyCli, IndexCommand, &[], |_| {
});
