use std::future::Future;

use clipanion_core::{CommandSpec, Info, State};

use crate::{details::{CommandExecutor, CommandExecutorAsync, CommandProvider, CommandResult}, format::Formatter};

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

enum PrepareResult<'a> {
    Success,
    Failure,
    Ready(State<'a>, &'a CommandSpec),
}

fn prepare_command<'a, S: CommandProvider>(builder: &'a clipanion_core::CliBuilder, env: &'a Environment) -> PrepareResult<'a> {
    let parse_result
        = builder.run(env.argv.iter().map(|s| s.as_str()));

    let Ok((state, command)) = parse_result else {
        println!("{}", Formatter::<S>::format_parse_error(&env.info, &parse_result.unwrap_err()));
        return PrepareResult::Failure;
    };

    PrepareResult::Ready(state, command)
}

fn finalize_command<S: CommandProvider>(env: &Environment, command_spec: &CommandSpec, command_result: CommandResult) -> std::process::ExitCode {
    if let Some(err) = &command_result.error_message {
        println!("{}", Formatter::<S>::format_error(&env.info, "Error", &err, vec![command_spec]));
    }

    command_result.exit_code
}

pub trait Cli {
    fn run(env: Environment) -> std::process::ExitCode;
    fn run_default() -> std::process::ExitCode;
}

impl<S: CommandProvider + CommandExecutor> Cli for S {
    fn run(env: Environment) -> std::process::ExitCode {
        let builder = S::build_cli()
            .unwrap();

        let preparation
            = prepare_command::<S>(&builder, &env);

        match preparation {
            PrepareResult::Success
                => std::process::ExitCode::SUCCESS,

            PrepareResult::Failure
                => std::process::ExitCode::FAILURE,

            PrepareResult::Ready(state, command_spec) => {
                let command_result
                    = S::execute_cli_state(&env, state);

                finalize_command::<S>(&env, &command_spec, command_result)
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

impl<S: CommandProvider + CommandExecutorAsync> CliAsync for S {
    async fn run(env: Environment) -> std::process::ExitCode {
        let builder = S::build_cli()
            .unwrap();

        let preparation
            = prepare_command::<S>(&builder, &env);

        match preparation {
            PrepareResult::Success
                => std::process::ExitCode::SUCCESS,

            PrepareResult::Failure
                => std::process::ExitCode::FAILURE,

            PrepareResult::Ready(state, command_spec) => {
                let command_result
                    = S::execute_cli_state(&env, state).await;

                finalize_command::<S>(&env, &command_spec, command_result)
            },
        }
    }

    async fn run_default() -> std::process::ExitCode {
        Self::run(Default::default()).await
    }
}
