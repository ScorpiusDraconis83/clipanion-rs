use clipanion::{prelude::*, program, test_cli_failure, test_cli_success, Error};

#[cli::command]
#[cli::path("foo")]
struct FooCommand {
    args: Vec<String>,
}

impl FooCommand {
    fn execute(&self) {
    }
}

#[cli::command]
#[cli::path("foo", "bar", "baz")]
struct BazCommand {
    args: Vec<String>,
}

impl BazCommand {
    fn execute(&self) {
    }
}

#[cli::command]
#[cli::path("foo", "bar")]
struct BarCommand {
    args: Vec<String>,
}

impl BarCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [FooCommand, BazCommand, BarCommand]);

test_cli_success!(it_works_with_foo, MyCli, FooCommand, &["foo"], |_| {
});

test_cli_success!(it_works_with_bar, MyCli, BarCommand, &["foo", "bar"], |_| {
});

test_cli_success!(it_works_with_baz, MyCli, BazCommand, &["foo", "bar", "baz"], |_| {
});
