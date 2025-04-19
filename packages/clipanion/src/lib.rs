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
macro_rules! program_provider {
    ($name:ident, [$($command:ty),* $(,)?]) => {
        pub struct $name {}

        impl $crate::details::CommandProvider for $name {
            fn command_usage(mut command_index: usize, opts: $crate::core::CommandUsageOptions) -> Result<$crate::core::CommandUsageResult, $crate::core::BuildError> {
                use $crate::details::CommandController;

                // We can't use recursive macros to generate match arms, and
                // it's not possible yet to get the index of the current
                // iteration in a `for` loop, so we have to use a manual
                // counter here and pray the compiler optimizes it.
                $({
                    if command_index == 0 {
                        return <$command>::command_usage(opts);
                    } else {
                        command_index -= 1;
                    }
                })*

                unreachable!();
            }

            fn register_to_cli_builder(builder: &mut $crate::core::CliBuilder) -> Result<(), $crate::core::BuildError> {
                use $crate::details::CommandController;

                $(<$command>::attach_command_to_cli(builder.add_command())?;)*

                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! program_executor {
    ($name:ident, [$($command:ty),* $(,)?]) => {
        impl $crate::details::CommandExecutor for $name {
            fn execute_cli_state(info: &$crate::advanced::Info, state: $crate::core::RunState) -> $crate::details::CommandResult {
                use $crate::details::CommandController;

                let mut command_index
                    = state.selected_index.unwrap();

                // We can't use recursive macros to generate match arms, and
                // it's not possible yet to get the index of the current
                // iteration in a `for` loop, so we have to use a manual
                // counter here and hope the compiler optimizes it.
                $({
                    if command_index == 0 {
                        let hydration_result
                            = <$command>::hydrate_command_from_state(info, state);
                        
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

    ($name:ident, [$($command:ty),* $(,)?], async) => {
        impl $crate::details::CommandExecutorAsync for $name {
            async fn execute_cli_state(info: &$crate::advanced::Info, state: $crate::core::RunState) -> $crate::details::CommandResult {
                use $crate::details::CommandController;

                let mut command_index
                    = state.selected_index.unwrap();

                // We can't use recursive macros to generate match arms, and
                // it's not possible yet to get the index of the current
                // iteration in a `for` loop, so we have to use a manual
                // counter here and hope the compiler optimizes it.
                $({
                    if command_index == 0 {
                        let hydration_result
                            = <$command>::hydrate_command_from_state(info, state);
                        
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
    ($name:ident, [$($command:ty),* $(,)?]) => {
        $crate::program_provider!($name, [$($command),*]);
        $crate::program_executor!($name, [$($command),*]);
    };
}

#[macro_export]
macro_rules! program_async {
    ($name:ident, [$($command:ty),* $(,)?]) => {
        $crate::program_provider!($name, [$($command),*]);
        $crate::program_executor!($name, [$($command),*], async);
    };
}
