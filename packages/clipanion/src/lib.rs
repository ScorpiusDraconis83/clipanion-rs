pub use clipanion_core as core;

extern crate clipanion_derive;

pub mod cli {
    pub use clipanion_derive::command;
}

pub mod advanced;
pub mod format;
pub mod details;

#[macro_export]
macro_rules! new {
    ($($command:ty),+ $(,)?) => { {
        struct CommandSet {};

        impl clipanion::details::CommandSet for CommandSet {
            fn command_usage(mut command_index: usize, opts: clipanion::core::CommandUsageOptions) -> Result<clipanion::core::CommandUsageResult, clipanion::core::BuildError> {
                use clipanion::details::CommandController;

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

            fn register_to_cli_builder(builder: &mut clipanion::core::CliBuilder) -> Result<(), clipanion::core::BuildError> {
                use clipanion::details::CommandController;

                $(<$command>::attach_command_to_cli(builder.add_command())?;)*

                Ok(())
            }

            fn execute_cli_state(info: &clipanion::advanced::Info, state: clipanion::core::RunState) -> clipanion::details::CommandResult {
                use clipanion::details::CommandController;

                let mut command_index = state.selected_index.unwrap();

                // We can't use recursive macros to generate match arms, and
                // it's not possible yet to get the index of the current
                // iteration in a `for` loop, so we have to use a manual
                // counter here and pray the compiler optimizes it.
                $({
                    if command_index == 0 {
                        let mut command = <$command>::default();
                        command.hydrate_command_from_state(state);
                        return command.execute().into();
                    } else {
                        command_index -= 1;
                    }
                })*

                std::unreachable!();
            }
        };

        clipanion::advanced::Cli::<CommandSet>::new()
    } };
}
