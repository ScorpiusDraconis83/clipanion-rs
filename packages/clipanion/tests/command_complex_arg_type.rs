use std::str::FromStr;

use clipanion::{prelude::*, test_cli_failure, test_cli_success, CommandError, Error};

#[derive(Debug, PartialEq, Eq)]
struct Size {
    pub width: usize,
    pub height: usize,
}

impl Size {
    fn new(width: usize, height: usize) -> Self {
        Size {width, height}
    }
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

        Ok(Size {width, height})
    }
}

#[cli::command(default)]
struct MyCommand {
    #[cli::option("--size", default = Size::new(0, 0))]
    size_opt: Size,

    size: Size,
}

impl MyCommand {
    fn execute(&self) {
    }
}

#[cli::program]
enum MyCli {
    MyCommand(MyCommand),
}

test_cli_success!(it_works, MyCli, MyCommand, &["10x20"], |command| {
    assert_eq!(command.size, Size {width: 10, height: 20});
});

test_cli_success!(it_works_with_option, MyCli, MyCommand, &["--size", "10x20", "30x40"], |command| {
    assert_eq!(command.size_opt, Size {width: 10, height: 20});
    assert_eq!(command.size, Size {width: 30, height: 40});
});

test_cli_failure!(it_fails_with_invalid_data, MyCli, &["10x"], |error| {
    assert_eq!(error, Error::CommandError(MyCommand::command_spec().unwrap(), CommandError::Custom("Invalid height".to_string())));
});
