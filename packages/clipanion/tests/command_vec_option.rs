use clipanion::{advanced::Environment, details::CommandProvider, prelude::*, program, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    #[cli::option("--my-option")]
    my_option: Vec<String>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_works, MyCli, MyCommand, &["--my-option", "foo", "--my-option", "baz"], |command| {
    assert_eq!(command.my_option, vec!["foo".to_string(), "baz".to_string()]);
});
