use clipanion::{prelude::*, program, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    #[cli::option("--foo")]
    value: Option<Option<(String, String)>>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_accepts_booleans, MyCli, MyCommand, &["--foo"], |command| {
    assert_eq!(command.value, Some(None));
});

test_cli_success!(it_accepts_arguments, MyCli, MyCommand, &["--foo", "foo", "bar"], |command| {
    assert_eq!(command.value, Some(Some(("foo".to_string(), "bar".to_string()))));
});
