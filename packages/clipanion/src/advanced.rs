use std::future::Future;

use clipanion_core::Info;

use crate::{details::{CommandExecutor, CommandExecutorAsync, CommandProvider}, format::Formatter};

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

        Self {
            argv: std::env::args().skip(1).collect(),
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

fn prepare_command<'cmds, 'args, S: CommandProvider>(builder: &'cmds clipanion_core::CliBuilder, env: &'args Environment) -> Result<clipanion_core::ParseResult<'cmds, 'args>, clipanion_core::Error<'cmds>> {
    let args
        = env.argv.iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>();

    builder.run(&args)
}

fn finalize_command<'cmds, 'args, S: CommandProvider>(env: &Environment, parse_result: Result<clipanion_core::ParseResult<'cmds, 'args>, clipanion_core::Error<'cmds>>) -> std::process::ExitCode {
    match parse_result {
        Ok(clipanion_core::ParseResult::Ready(_, _)) => {
            unreachable!("This is supposed to be handled by the executor");
        },

        Ok(clipanion_core::ParseResult::Builtin(_)) => {
            std::process::ExitCode::SUCCESS
        },

        Err(clipanion_core::Error::CommandError(command_spec, command_error)) => {
            println!("{}", Formatter::<S>::format_error(&env.info, "Error", &command_error.to_string(), vec![command_spec]));
            std::process::ExitCode::FAILURE
        },

        Err(clipanion_core::Error::AmbiguousSyntax(_)) => {
            std::process::ExitCode::FAILURE
        },

        Err(clipanion_core::Error::InternalError) => {
            std::process::ExitCode::FAILURE
        },

        Err(clipanion_core::Error::NotFound(_)) => {
            std::process::ExitCode::FAILURE
        },
    }
}

pub trait Cli {
    fn run(env: Environment) -> std::process::ExitCode;
    fn run_default() -> std::process::ExitCode;
}

impl<S: CommandProvider + CommandExecutor> Cli for S {
    fn run(env: Environment) -> std::process::ExitCode {
        let builder = S::build_cli()
            .unwrap();

        let parse_result
            = prepare_command::<S>(&builder, &env);

        if let Ok(clipanion_core::ParseResult::Ready(state, command_spec)) = parse_result {
            let command_result
                = S::execute_cli_state(&env, state);

            if let Some(error_message) = &command_result.error_message {
                finalize_command::<S>(&env, Err(clipanion_core::Error::CommandError(command_spec, error_message.to_string().into())));
            }

            command_result.exit_code
        } else {
            finalize_command::<S>(&env, parse_result)
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

impl<S: CommandProvider + CommandExecutorAsync> CliAsync for S {
    async fn run(env: Environment) -> std::process::ExitCode {
        let builder = S::build_cli()
            .unwrap();

        let parse_result
            = prepare_command::<S>(&builder, &env);

        if let Ok(clipanion_core::ParseResult::Ready(state, command_spec)) = parse_result {
            let command_result
                = S::execute_cli_state(&env, state).await;

            if let Some(error_message) = &command_result.error_message {
                finalize_command::<S>(&env, Err(clipanion_core::Error::CommandError(command_spec, error_message.to_string().into())));
            }

            command_result.exit_code
        } else {
            finalize_command::<S>(&env, parse_result)
        }
    }

    async fn run_default() -> std::process::ExitCode {
        Self::run(Default::default()).await
    }
}
