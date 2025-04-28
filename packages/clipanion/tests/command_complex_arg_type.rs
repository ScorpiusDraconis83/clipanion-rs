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

#[cli::command(default)]
struct MyCommand {
    size: Size,
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

test_cli_success!(it_works, MyCli, MyCommand, &["10x20"], |command| {
    assert_eq!(command.size, Size {_width: 10, _height: 20});
});

test_cli_failure!(it_fails_with_invalid_data, MyCli, MyCommand, &["10x"], |error| {
    assert_eq!(error, Error::CommandError(MyCommand::command_spec().unwrap(), CommandError::Custom("Invalid height".to_string())));
});
