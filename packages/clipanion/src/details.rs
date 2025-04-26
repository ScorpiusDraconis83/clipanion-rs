use std::{convert::Infallible, error::Error, future::Future, iter::Peekable};

use clipanion_core::{CommandError, CommandSpec, CustomError};

use crate::advanced::Environment;

pub fn handle_parse_error<E: Error + 'static>(err: E) -> CustomError {
    match std::any::TypeId::of::<E>() == std::any::TypeId::of::<Infallible>() {
        true => unreachable!("Infallible error occurred"),
        false => CustomError::new(err.to_string()),
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

impl From<CommandError> for CommandResult {
    fn from(error: CommandError) -> Self {
        Self {
            exit_code: std::process::ExitCode::FAILURE,
            error_message: Some(error.to_string()),
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
impl<T: Into<CommandResult>, E: std::fmt::Display> From<Result<T, E>> for CommandResult {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(value) => value.into(),

            Err(err) => {
                let message = format!("{:#}", err);
                let maybe_message = if message.is_empty() { None } else { Some(message) };

                Self {
                    exit_code: std::process::ExitCode::FAILURE,
                    error_message: maybe_message,
                }
            },
        }
    }
}

/**
 * Internal trait implemented by the #[command] attribute.
 */
pub trait CommandController {
    fn command_usage(opts: clipanion_core::CommandUsageOptions) -> Result<clipanion_core::CommandUsageResult, clipanion_core::BuildError>;
    fn command_spec() -> Result<CommandSpec, clipanion_core::BuildError>;
    fn hydrate_command_from_state(environment: &Environment, state: &clipanion_core::State) -> Result<Self, clipanion_core::CommandError> where Self: Sized;
}

/**
 * Internal traits implemented by the `program!` and `program_async!` macros. Used
 * to statically aggregate multiple commands together in a single `Cli` type.
 */
pub trait CommandProvider {
    fn command_usage(command_index: usize, opts: clipanion_core::CommandUsageOptions) -> Result<clipanion_core::CommandUsageResult, clipanion_core::BuildError>;
    fn parse_args<'a, 'b>(builder: &'a clipanion_core::CliBuilder, environment: &'b Environment) -> Result<Self, clipanion_core::Error<'a>> where Self: Sized;
    fn build_cli() -> Result<clipanion_core::CliBuilder, clipanion_core::BuildError>;
}

pub trait CommandExecutor {
    fn execute_cli_state<'a>(env: &Environment, state: clipanion_core::State<'a>) -> crate::details::CommandResult;
}

pub trait CommandExecutorAsync {
    fn execute_cli_state<'a>(env: &Environment, state: clipanion_core::State<'a>) -> impl Future<Output = crate::details::CommandResult>;
}

pub fn cautious_take_if<T: Iterator>(it: &mut Peekable<T>, check: impl FnOnce(&T::Item) -> bool) -> Option<T::Item> {
    if let Some(item) = it.peek() {
        if check(item) {
            return it.next();
        }
    }

    None
}
