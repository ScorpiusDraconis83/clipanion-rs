use clipanion::{prelude::*, test_cli_success};

#[cli::command(default)]
struct MyCommand {
    #[cli::option("--my-option")]
    my_option: Vec<(String, String)>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["--my-option", "foo", "bar", "--my-option", "baz", "qux"], |command| {
    assert_eq!(command.my_option, vec![("foo".to_string(), "bar".to_string()), ("baz".to_string(), "qux".to_string())]);
});
