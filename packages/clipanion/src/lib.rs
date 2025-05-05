pub extern crate clipanion_derive;

pub use clipanion_core as core;
pub use clipanion_derive as derive;

pub mod cli {
    pub use clipanion_derive::command;
}

pub mod advanced;
pub mod format;
pub mod details;
pub mod prelude;

pub use advanced::Environment;

pub use clipanion_core::{
    CommandError,
    Error,
};

#[macro_export]
macro_rules! program_enum {
    ($name:ident, [$($command:ty),* $(,)?]) => {
        #[clipanion::derive::cli_enum($($command),*)]
        #[clipanion::derive::cli_exec_sync($($command),*)]
        enum $name {}
    };

    ($name:ident, [$($command:ty),* $(,)?], async) => {
        #[clipanion::derive::cli_enum($($command),*)]
        #[clipanion::derive::cli_exec_async($($command),*)]
        enum $name {}
    };
}

#[macro_export]
macro_rules! program_provider {
    ($name:ident, [$($command:ty),* $(,)?]) => {
        impl $crate::details::CommandProvider for $name {
            type Command = $name;

            fn command_usage(command_index: usize, opts: $crate::core::CommandUsageOptions) -> Result<$crate::core::CommandUsageResult, $crate::core::BuildError> {
                use $crate::details::CommandController;

                const FNS: &[fn($crate::core::CommandUsageOptions) -> Result<$crate::core::CommandUsageResult, $crate::core::BuildError>] = &[
                    $(<$command>::command_usage),*
                ];

                FNS[command_index](opts)
            }

            fn parse_args<'args>(builder: &$crate::core::CliBuilder<'static>, environment: &'args $crate::advanced::Environment) -> Result<$crate::core::SelectionResult<'static, $name>, $crate::core::Error<'args>> {
                let argv
                    = environment.argv.iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>();

                let parse_result
                    = builder.run(&argv)?;

                if let $crate::core::ParseResult::Builtin(builtin) = parse_result {
                    return Ok($crate::core::SelectionResult::Builtin(builtin));
                }

                let $crate::core::ParseResult::Selector(selector) = parse_result else {
                    unreachable!("Expected a selector result");
                };

                const FNS: &[fn(&$crate::advanced::Environment, &$crate::core::State<'_>) -> Result<$name, $crate::core::CommandError>] = &[
                    $(|environment, state| {
                        use $crate::details::CommandController;

                        let command
                            = <$command>::hydrate_command_from_state(environment, state)?;

                        Ok(command.into())
                    }),*
                ];

                let hydration_results
                    = selector.states.iter()
                        .map(|state| {
                            let command_spec
                                = selector.commands[state.context_id];

                            let f
                                = FNS[state.context_id];

                            let result = f(environment, &state)
                                .map_err(|e| $crate::core::Error::CommandError(command_spec, e))?;

                            Ok(result)
                        })
                        .collect::<Vec<_>>();

                selector.get_best_hydrated_state(hydration_results)
            }

            fn build_cli() -> Result<$crate::core::CliBuilder<'static>, $crate::core::BuildError> {
                use $crate::details::CommandController;

                let mut builder
                    = $crate::core::CliBuilder::new();

                $(builder.add_command(<$command>::command_spec()?);)*

                Ok(builder)
            }
        }
    };
}

#[macro_export]
macro_rules! program {
    ($name:ident, [$($command:ty),* $(,)?]) => {
        $crate::program_enum!($name, [$($command),*]);
        $crate::program_provider!($name, [$($command),*]);
    };
}

#[macro_export]
macro_rules! program_async {
    ($name:ident, [$($command:ty),* $(,)?]) => {
        $crate::program_enum!($name, [$($command),*], async);
        $crate::program_provider!($name, [$($command),*]);
    };
}

#[macro_export]
macro_rules! test_cli_success {
    ($test_name:ident, $cli_name:ident, $command_name:ty, $args:expr, $fn:expr) => {
        #[test]
        fn $test_name() {
            const ARGS: &[&str] = $args;

            let cli = $cli_name::build_cli().unwrap();
            let env = $crate::advanced::Environment::default().with_argv(ARGS.iter().map(|s| s.to_string()).collect());

            println!("cli: {:?}", cli.compile());

            let result = $cli_name::parse_args(&cli, &env);
            let f: fn($command_name) -> () = $fn;

            let result = result
                .expect("expected command, got error");

            let $crate::core::SelectionResult::Command(command, _) = result else {
                unreachable!("expected command, got error");
            };

            f(command.into());
        }
    };
}

#[macro_export]
macro_rules! test_cli_failure {
    ($test_name:ident, $cli_name:ident, $command_name:ty, $args:expr, $fn:expr) => {
        #[test]
        fn $test_name() {
            const ARGS: &[&str] = $args;

            let cli = $cli_name::build_cli().unwrap();
            let env = $crate::advanced::Environment::default().with_argv(ARGS.iter().map(|s| s.to_string()).collect());

            println!("cli: {:?}", cli.compile());

            let result = $cli_name::parse_args(&cli, &env);
            let f: fn($crate::core::Error<'_>) -> () = $fn;

            f(match result {
                Err(error) => error,
                Ok(_) => panic!("expected error"),
            });
        }
    };
}
