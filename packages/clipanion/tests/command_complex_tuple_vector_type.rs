use std::str::FromStr;

use clipanion::{prelude::*, program, test_cli_failure, test_cli_success, CommandError, Error};

#[derive(Debug, PartialEq, Eq)]
struct Size {
    pub _width: usize,
    pub _height: usize,
}

impl FromStr for Size {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('x').collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err("Invalid size".to_string());
        }

        let width
            = parts[0].parse::<usize>().map_err(|_| "Invalid width")?;
        let height
            = parts[1].parse::<usize>().map_err(|_| "Invalid height")?;

        Ok(Size {_width: width, _height: height})
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Color {
    Red,
    Green,
    Blue,
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "blue" => Ok(Color::Blue),
            _ => Err("Invalid color".to_string()),
        }
    }
}

#[cli::command(default)]
struct MyCommand {
    #[cli::option("--size")]
    size: (Size, Color),
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_works, MyCli, MyCommand, &["--size", "10x20", "red"], |command| {
    assert_eq!(command.size, (Size {_width: 10, _height: 20}, Color::Red));
});

test_cli_failure!(it_fails_with_invalid_data_1, MyCli, MyCommand, &["--size", "10x", "red"], |error| {
    assert_eq!(error, Error::CommandError(MyCommand::command_spec().unwrap(), CommandError::Custom("Invalid height".to_string())));
});

test_cli_failure!(it_fails_with_invalid_data_2, MyCli, MyCommand, &["--size", "10x20", "salt"], |error| {
    assert_eq!(error, Error::CommandError(MyCommand::command_spec().unwrap(), CommandError::Custom("Invalid color".to_string())));
});
