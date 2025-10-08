use clipanion::{prelude::*, test_cli_success};

#[cli::command]
#[cli::path("foo", "bar")]
struct MyCommand {
    #[cli::option("--my-option")]
    my_option: Option<bool>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["foo", "bar", "--my-option"], |command| {
    assert_eq!(command.my_option, Some(true));
});

test_cli_success!(it_works_if_the_option_is_first, MyCli, MyCommand, &["--my-option", "foo", "bar"], |command| {
    assert_eq!(command.my_option, Some(true));
});

test_cli_success!(it_works_if_the_option_is_between_the_path_segments, MyCli, MyCommand, &["foo", "--my-option", "bar"], |command| {
    assert_eq!(command.my_option, Some(true));
});
