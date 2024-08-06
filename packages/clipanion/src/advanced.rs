use clipanion_core::HELP_COMMAND_INDEX;

use crate::{details::CommandSet, format::Formatter};

/**
 * Used to define the properties of the CLI. In general you can ignore this and
 * just use the `run_with_default()` function instead.
 */
pub struct Info {
    pub argv: Vec<String>,
    pub program_name: String,
    pub binary_name: String,
    pub version: String,
    pub about: String,
    pub colorized: bool,
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

pub struct Cli<S> {
    builder: clipanion_core::CliBuilder,
    phantom: std::marker::PhantomData<S>,
}

impl<S: CommandSet> Cli<S> {
    pub fn new() -> Self {
        let mut cli = Self {
            builder: clipanion_core::CliBuilder::new(),
            phantom: Default::default(),
        };

        S::register_to_cli_builder(&mut cli.builder)
            .unwrap();

        cli
    }

    pub fn run(&self, info: Info) -> std::process::ExitCode {
        let machine = self.builder.compile();

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

    pub fn run_default(&self) -> std::process::ExitCode {
        self.run(Default::default())
    }
}
