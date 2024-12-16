use std::fmt::Display;

use crate::advanced::Info;

/**
 * Internal error type that may be emitted during `hydrate_command_from_state`
 * if the command is unable to hydrate itself from the state (usually because
 * the `try_into` implementations failed to convert the input string into the
 * expected value type).
 */
pub struct HydrationError {
    pub message: String,
}

impl HydrationError {
    pub fn new<T: Display>(source: T) -> Self {
        Self {
            message: source.to_string(),
        }
    }
}

/**
 * Internal trait used to convert whatever the `execute()` function returns
 * into an exit code. It makes it easier to return `()` from simple commands
 * without having to return a specific number.
 */
#[derive(Debug)]
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

impl From<HydrationError> for CommandResult {
    fn from(error: HydrationError) -> Self {
        Self {
            exit_code: std::process::ExitCode::FAILURE,
            error_message: Some(error.message),
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

impl From<std::process::ExitStatus> for CommandResult {
    fn from(exit_status: std::process::ExitStatus) -> Self {
        Self {
            exit_code: std::process::ExitCode::from(exit_status.code().unwrap_or(1) as u8),
            error_message: None,
        }
    }
}

#[cfg(feature = "anyhow")]
impl<T: Into<CommandResult>> From<Result<T, anyhow::Error>> for CommandResult {
    fn from(value: Result<T, anyhow::Error>) -> Self {
        match value {
            Ok(value) => value.into(),

            Err(err) => Self {
                exit_code: std::process::ExitCode::FAILURE,
                error_message: Some(format!("{:?}", err)),
            },
        }
    }
}

#[cfg(not(feature = "anyhow"))]
impl<T: Into<CommandResult>, E: Display> From<Result<T, E>> for CommandResult {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(value) => value.into(),

            Err(err) => Self {
                exit_code: std::process::ExitCode::FAILURE,
                error_message: Some(format!("{:#}", err)),
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
    fn hydrate_command_from_state(&mut self, info: &Info, state: clipanion_core::RunState) -> Result<(), HydrationError>;
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
