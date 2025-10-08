use clipanion::{prelude::*, program, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    #[cli::option("--my-option")]
    value: Option<String>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works_with_value, MyCli, MyCommand, &["--my-option", "foo"], |command| {
    assert_eq!(command.value, Some("foo".to_string()));
});

test_cli_success!(it_supports_bindings, MyCli, MyCommand, &["--my-option=foo"], |command| {
    assert_eq!(command.value, Some("foo".to_string()));
});

test_cli_success!(it_is_optional, MyCli, MyCommand, &[], |command| {
    assert_eq!(command.value, None);
});
