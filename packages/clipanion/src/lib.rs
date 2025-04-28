pub use clipanion_core as core;

extern crate clipanion_derive;

pub mod cli {
    pub use clipanion_derive::command;
}

pub mod advanced;
pub mod format;
pub mod details;
pub mod prelude;

#[macro_export]
macro_rules! program_enum {
    ($name:ident, [$($command:ident),* $(,)?]) => {
        pub enum $name {
            $($command($command),)*
        }
    }
}

#[macro_export]
macro_rules! program_provider {
    ($name:ident, [$($command:ident),* $(,)?]) => {
        impl $crate::details::CommandProvider for $name {
            fn command_usage(command_index: usize, opts: $crate::core::CommandUsageOptions) -> Result<$crate::core::CommandUsageResult, $crate::core::BuildError> {
                use $crate::details::CommandController;

                const FNS: &[fn($crate::core::CommandUsageOptions) -> Result<$crate::core::CommandUsageResult, $crate::core::BuildError>] = &[
                    $(<$command>::command_usage),*
                ];

                FNS[command_index](opts)
            }

            fn parse_args<'cmds>(builder: &$crate::core::CliBuilder<'cmds>, environment: &$crate::advanced::Environment) -> Result<$name, $crate::core::Error<'cmds>> {
                let argv
                    = environment.argv.iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>();

                let (state, command_spec)
                    = builder.run(&argv)?;

                const FNS: &[fn(&$crate::advanced::Environment, &$crate::core::State<'_>) -> Result<$name, $crate::core::CommandError>] = &[
                    $(|environment, state| {
                        use $crate::details::CommandController;

                        let command
                            = <$command>::hydrate_command_from_state(environment, state)?;

                        Ok($name::$command(command))
                    }),*
                ];

                let result = FNS[state.context_id](environment, &state)
                    .map_err(|e| $crate::core::Error::CommandError(command_spec, e))?;

                Ok(result)
            }

            fn build_cli() -> Result<$crate::core::CliBuilder<'static>, $crate::core::BuildError> {
                use $crate::details::CommandController;

                let mut builder
                    = $crate::core::CliBuilder::new();

                $(builder.add_command(<$command>::command_spec()?);)*

                Ok(builder)
            }
        }
    }
}

#[macro_export]
macro_rules! program_executor {
    ($name:ident, [$($command:ident),* $(,)?]) => {
        impl $crate::details::CommandExecutor for $name {
            fn execute_cli_state(environment: &$crate::advanced::Environment, state: $crate::core::State) -> $crate::details::CommandResult {
                use $crate::details::CommandController;

                let mut command_index
                    = state.context_id;

                // We can't use recursive macros to generate match arms, and
                // it's not possible yet to get the index of the current
                // iteration in a `for` loop, so we have to use a manual
                // counter here and hope the compiler optimizes it.
                $({
                    if command_index == 0 {
                        let hydration_result
                            = <$command>::hydrate_command_from_state(&environment, &state);
                        
                        let command = match hydration_result {
                            Err(hydration_error) => return hydration_error.into(),
                            Ok(command) => command
                        };

                        let command_result
                            = command.execute();

                        return command_result.into();
                    } else {
                        command_index -= 1;
                    }
                })*

                std::unreachable!();
            }
        }
    };

    ($name:ident, [$($command:ident),* $(,)?], async) => {
        impl $crate::details::CommandExecutorAsync for $name {
            async fn execute_cli_state<'a>(environment: &$crate::advanced::Environment, state: $crate::core::State<'a>) -> $crate::details::CommandResult {
                use $crate::details::CommandController;

                let mut command_index
                    = state.context_id;

                // We can't use recursive macros to generate match arms, and
                // it's not possible yet to get the index of the current
                // iteration in a `for` loop, so we have to use a manual
                // counter here and hope the compiler optimizes it.
                $({
                    if command_index == 0 {
                        let hydration_result
                            = <$command>::hydrate_command_from_state(&environment, &state);
                        
                        let command = match hydration_result {
                            Err(hydration_error) => return hydration_error.into(),
                            Ok(command) => command
                        };

                        let command_result
                            = command.execute().await;

                        return command_result.into();
                    } else {
                        command_index -= 1;
                    }
                })*

                std::unreachable!();
            }
        }
    };
}

#[macro_export]
macro_rules! program {
    ($name:ident, [$($command:ident),* $(,)?]) => {
        $crate::program_enum!($name, [$($command),*]);
        $crate::program_provider!($name, [$($command),*]);
        $crate::program_executor!($name, [$($command),*]);
    };
}

#[macro_export]
macro_rules! program_async {
    ($name:ident, [$($command:ident),* $(,)?]) => {
        $crate::program_enum!($name, [$($command),*]);
        $crate::program_provider!($name, [$($command),*]);
        $crate::program_executor!($name, [$($command),*], async);
    };
}

#[macro_export]
macro_rules! test_cli_success {
    ($test_name:ident, $cli_name:ident, $command_name:ident, $args:expr, $fn:expr) => {
        #[test]
        fn $test_name() {
            const ARGS: &[&str] = $args;

            let cli = $cli_name::build_cli().unwrap();
            let env = Environment::default().with_argv(ARGS.iter().map(|s| s.to_string()).collect());

            let result = $cli_name::parse_args(&cli, &env);
            let f: fn($command_name) -> () = $fn;

            f(match result {
                Ok($cli_name::$command_name(command)) => command,
                Ok(_) => panic!("expected $command_name"),
                Err(error) => panic!("expected command, got error: {}", error),
            });
        }
    };
}

#[macro_export]
macro_rules! test_cli_failure {
    ($test_name:ident, $cli_name:ident, $command_name:ident, $args:expr, $fn:expr) => {
        #[test]
        fn $test_name() {
            const ARGS: &[&str] = $args;

            let cli = $cli_name::build_cli().unwrap();
            let env = Environment::default().with_argv(ARGS.iter().map(|s| s.to_string()).collect());

            let result = $cli_name::parse_args(&cli, &env);
            let f: fn($crate::core::Error<'_>) -> () = $fn;

            f(match result {
                Err(error) => error,
                Ok(_) => panic!("expected error"),
            });
        }
    };
}
