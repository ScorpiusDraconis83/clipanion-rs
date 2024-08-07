use std::fmt::Display;

use crate::advanced::Info;

/**
 * Internal trait used to convert whatever the `execute()` function returns
 * into an exit code. It makes it easier to return `()` from simple commands
 * without having to return a specific number.
 */
pub struct CommandResult {
    pub exit_code: std::process::ExitCode,
    pub error_message: Option<String>,
}

impl From<()> for CommandResult {
    fn from(_: ()) -> Self {
        Self {
            exit_code: std::process::ExitCode::SUCCESS,
            error_message: None,
        }
    }
}

impl From<std::process::ExitCode> for CommandResult {
    fn from(exit_code: std::process::ExitCode) -> Self {
        Self {
            exit_code,
            error_message: None,
        }
    }
}

impl<T, E> From<Result<T, E>> for CommandResult where E: Display {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(_) => Self {
                exit_code: std::process::ExitCode::SUCCESS,
                error_message: None,
            },

            Err(err) => Self {
                exit_code: std::process::ExitCode::FAILURE,
                error_message: Some(format!("{}", err)),
            },
        }
    }
}

/**
 * Internal trait implemented by the #[command] attribute.
 */
pub trait CommandController {
    fn command_usage(opts: clipanion_core::CommandUsageOptions) -> Result<clipanion_core::CommandUsageResult, clipanion_core::BuildError>;
    fn attach_command_to_cli(builder: &mut clipanion_core::CommandBuilder) -> Result<(), clipanion_core::BuildError>;
    fn hydrate_command_from_state(&mut self, info: &Info, state: clipanion_core::RunState);
}

/**
 * Internal trait implemented by the `new!` macro. Used to statically aggregate
 * multiple commands together in a single type.
 */
pub trait CommandSet {
    fn command_usage(command_index: usize, opts: clipanion_core::CommandUsageOptions) -> Result<clipanion_core::CommandUsageResult, clipanion_core::BuildError>;
    fn register_to_cli_builder(builder: &mut clipanion_core::CliBuilder) -> Result<(), clipanion_core::BuildError>;
    fn execute_cli_state(info: &Info, state: clipanion_core::RunState) -> crate::details::CommandResult;
}
