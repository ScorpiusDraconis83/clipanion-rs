use clipanion_core::HELP_COMMAND_INDEX;

use crate::{details::CommandSet, format::Formatter};

/**
 * Used to define the properties of the CLI. In general you can ignore this and
 * just use the `run_with_default()` function instead.
 */

#[derive(Debug, Clone)]
pub struct Info {
    pub argv: Vec<String>,
    pub program_name: String,
    pub binary_name: String,
    pub version: String,
    pub about: String,
    pub colorized: bool,
}

impl Info {
    pub fn with_argv(&self, argv: Vec<String>) -> Self {
        Self {argv, ..self.clone()}
    }
}

impl Default for Info {
    fn default() -> Self {
        let binary_name = std::env::args()
            .next().unwrap()
            .split('/').last().unwrap()
            .to_string();

        Self {
            argv: std::env::args().skip(1).collect(),
            program_name: env!("CARGO_PKG_NAME").to_string(),
            binary_name,
            version: env!("CARGO_PKG_VERSION").to_string(),
            about: env!("CARGO_PKG_DESCRIPTION").to_string(),
            colorized: true,
        }
    }
}

pub trait Cli {
    fn run(info: Info) -> std::process::ExitCode;
    fn run_default() -> std::process::ExitCode;
}

impl<S: CommandSet> Cli for S {
    fn run(info: Info) -> std::process::ExitCode {
        let mut builder = clipanion_core::CliBuilder::new();

        S::register_to_cli_builder(&mut builder)
            .unwrap();

        let machine = builder.compile();

        let parse_result
            = clipanion_core::run_machine(&machine, &info.argv);

        if let Err(parse_error) = parse_result {
            println!("{}", Formatter::<S>::format_parse_error(&info, &parse_error));
            return std::process::ExitCode::FAILURE;
        }

        let parse_state = parse_result.unwrap();

        if parse_state.selected_index == Some(HELP_COMMAND_INDEX) {
            println!("TODO: Show the help message");
            return std::process::ExitCode::SUCCESS;
        }

        let command_index = parse_state.selected_index.unwrap();
        let command_result = S::execute_cli_state(&info, parse_state);

        if let Some(err) = &command_result.error_message {
            println!("{}", Formatter::<S>::format_error(&info, "Error", &err, &[command_index]));
        }

        command_result.exit_code
    }

    fn run_default() -> std::process::ExitCode {
        Self::run(Default::default())
    }
}
