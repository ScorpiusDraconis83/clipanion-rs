use clipanion::{prelude::*, program, test_cli_success};

#[cli::command(default)]
pub struct MyCommand {
    #[cli::option("--my-option", default = false)]
    value_default_false: bool,

    #[cli::option("--my-option-default-true", default = true)]
    value_default_true: bool,

    #[cli::option("--my-option-default-none")]
    value_default_none: Option<bool>,

    #[cli::option("--my-option-string")]
    value_string: Option<String>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_supports_negated_options, MyCli, MyCommand, &["--no-my-option"], |command| {
    assert_eq!(command.value_default_false, false);
});

test_cli_success!(it_supports_negated_options_override_false, MyCli, MyCommand, &["--my-option", "--no-my-option"], |command| {
    assert_eq!(command.value_default_false, false);
});

test_cli_success!(it_supports_negated_options_override_true, MyCli, MyCommand, &["--my-option", "--no-my-option", "--my-option"], |command| {
    assert_eq!(command.value_default_false, true);
});

test_cli_success!(it_supports_negated_options_default_true, MyCli, MyCommand, &["--no-my-option-default-true"], |command| {
    assert_eq!(command.value_default_true, false);
});

test_cli_success!(it_supports_negated_options_default_none, MyCli, MyCommand, &["--no-my-option-default-none"], |command| {
    assert_eq!(command.value_default_none, Some(false));
});

test_cli_success!(it_supports_negated_options_string, MyCli, MyCommand, &["--my-option-string", "foo", "--no-my-option-string"], |command| {
    assert_eq!(command.value_string, None);
});
