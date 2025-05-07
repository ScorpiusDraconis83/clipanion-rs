use std::str::FromStr;

use clipanion::{prelude::*, program, test_cli_success};

#[derive(Debug, PartialEq, Eq)]
struct Foo;

#[derive(Debug, PartialEq, Eq)]
struct Bar;

impl FromStr for Foo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "foo" {
            Ok(Foo)
        } else {
            Err("Invalid foo".to_string())
        }
    }
}

impl FromStr for Bar {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "bar" {
            Ok(Bar)
        } else {
            Err("Invalid bar".to_string())
        }
    }
}

#[cli::command(default)]
struct FooCommand {
    foo: Foo,
}

#[cli::command(default)]
struct BarCommand {
    bar: Bar,
}

impl FooCommand {
    fn execute(&self) {
    }
}

impl BarCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [FooCommand, BarCommand]);

test_cli_success!(it_works_with_foo, MyCli, FooCommand, &["foo"], |command| {
    assert_eq!(command.foo, Foo);
});

test_cli_success!(it_works_with_bar, MyCli, BarCommand, &["bar"], |command| {
    assert_eq!(command.bar, Bar);
});
