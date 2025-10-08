pub extern crate clipanion_derive;

pub use clipanion_core as core;
pub use clipanion_derive as derive;

pub mod cli {
    pub use clipanion_derive::command;
    pub use clipanion_derive::program;
}

pub mod advanced;
pub mod format;
pub mod details;
pub mod prelude;

pub use advanced::Environment;

pub use clipanion_core::{
    BuiltinCommand,
    CommandError,
    Error,
};

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
                .unwrap_or_else(|err| panic!("expected command, got error: {:?}", err));

            let command = match result {
                $crate::core::SelectionResult::Builtin(builtin) => {
                    panic!("expected command, got builtin: {:?}", builtin);
                },

                $crate::core::SelectionResult::Command(_, _, command) => {
                    command
                },
            };

            let command
                = <$cli_name as $crate::details::CliEnums>::Enum::try_from(command)
                    .unwrap();

            f(command.into());
        }
    };
}

#[macro_export]
macro_rules! test_cli_failure {
    ($test_name:ident, $cli_name:ident, $args:expr, $fn:expr) => {
        #[test]
        fn $test_name() {
            const ARGS: &[&str] = $args;

            let cli = $cli_name::build_cli().unwrap();
            let env = $crate::advanced::Environment::default()
                .with_argv(ARGS.iter().map(|s| s.to_string()).collect());

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
