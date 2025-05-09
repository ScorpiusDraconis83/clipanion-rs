use std::future::Future;

use clipanion_core::{Info, SelectionResult};

use crate::{details::{CliEnums, CommandExecutor, CommandExecutorAsync, CommandProvider}, format::Formatter};

/**
 * Used to define the properties of the CLI. In general you can ignore this and
 * just use the `run_with_default()` function instead.
 */

 #[derive(Debug, Clone)]
 pub struct Environment {
    pub info: Info,
    pub argv: Vec<String>,
 }
 
impl Environment {
    pub fn with_argv(self, argv: Vec<String>) -> Self {
        Self {argv, ..self.clone()}
    }
}

impl Default for Environment {
    fn default() -> Self {
        let binary_name = std::env::args()
            .next().unwrap()
            .split('/').last().unwrap()
            .to_string();

        let argv
            = std::env::args()
                .skip(1)
                .collect();

        Self {
            argv,
            info: Info {
                program_name: env!("CARGO_PKG_NAME").to_string(),
                binary_name,
                version: env!("CARGO_PKG_VERSION").to_string(),
                about: env!("CARGO_PKG_DESCRIPTION").to_string(),
                colorized: true,
            },
        }
    }
}

fn report_error<'cmds, 'args, S: CommandProvider>(env: &Environment, err: clipanion_core::Error<'cmds>) -> std::process::ExitCode {
    match err {
        clipanion_core::Error::CommandError(command_spec, command_error) => {
            println!("{}", Formatter::<S>::format_error(&env.info, "Error", &command_error.to_string(), vec![command_spec]));
            std::process::ExitCode::FAILURE
        },

        clipanion_core::Error::AmbiguousSyntax(command_specs) => {
            println!("{}", Formatter::<S>::format_error(&env.info, "Error", &"The provided arguments are ambiguous and need to be refined further. Possible options are:", command_specs));
            std::process::ExitCode::FAILURE
        },

        clipanion_core::Error::InternalError => {
            std::process::ExitCode::FAILURE
        },

        clipanion_core::Error::NotFound(command_specs) => {
            println!("{}", Formatter::<S>::format_error(&env.info, "Error", &"The provided arguments don't match any known syntax; use `--help` to get a list of possible options", command_specs));
            std::process::ExitCode::FAILURE
        },
    }
}

pub trait Cli {
    fn run(env: Environment) -> std::process::ExitCode;
    fn run_default() -> std::process::ExitCode;
}

impl<S> Cli for S where S: CliEnums + CommandProvider, S::Enum: CommandExecutor {
    fn run(env: Environment) -> std::process::ExitCode {
        let builder = S::build_cli()
            .unwrap();

        let parse_result
            = S::parse_args(&builder, &env);

        match parse_result {
            Ok(SelectionResult::Builtin(_command)) => {
                todo!()
            },

            Ok(SelectionResult::Command(command_spec, _, partial_command)) => {
                let full_command = match <S as CliEnums>::Enum::try_from(partial_command) {
                    Ok(full_command)
                        => full_command,

                    Err(err)
                        => return report_error::<S>(&env, clipanion_core::Error::CommandError(command_spec, err)),
                };

                let command_result
                    = full_command.execute(&env);

                if let Some(error_message) = &command_result.error_message {
                    return report_error::<S>(&env, clipanion_core::Error::CommandError(command_spec, error_message.to_string().into()));
                }

                command_result.exit_code
            },

            Err(err) => {
                report_error::<S>(&env, err)
            },
        }
    }

    fn run_default() -> std::process::ExitCode {
        Self::run(Default::default())
    }
}

pub trait CliAsync {
    fn run(env: Environment) -> impl Future<Output = std::process::ExitCode>;
    fn run_default() -> impl Future<Output = std::process::ExitCode>;
}

impl<S> CliAsync for S where S: CliEnums + CommandProvider, S::Enum: CommandExecutorAsync {
    async fn run(env: Environment) -> std::process::ExitCode {
        let builder = S::build_cli()
            .unwrap();

        let parse_result
            = S::parse_args(&builder, &env);

        match parse_result {
            Ok(SelectionResult::Builtin(_command)) => {
                todo!()
            },

            Ok(SelectionResult::Command(command_spec, _, partial_command)) => {
                let full_command = match <S as CliEnums>::Enum::try_from(partial_command) {
                    Ok(full_command)
                        => full_command,

                    Err(err)
                        => return report_error::<S>(&env, clipanion_core::Error::CommandError(command_spec, err)),
                };

                let command_result
                    = full_command.execute(&env).await;

                if let Some(error_message) = &command_result.error_message {
                    return report_error::<S>(&env, clipanion_core::Error::CommandError(command_spec, error_message.to_string().into()));
                }

                command_result.exit_code
            },

            Err(err) => {
                report_error::<S>(&env, err)
            },
        }
    }

    async fn run_default() -> std::process::ExitCode {
        Self::run(Default::default()).await
    }
}
