use clipanion::{prelude::*, program, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    #[cli::option("--my-option")]
    my_option: Option<(String, String)>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_works, MyCli, MyCommand, &["--my-option", "foo", "bar"], |command| {
    assert_eq!(command.my_option, Some(("foo".to_string(), "bar".to_string())));
});
