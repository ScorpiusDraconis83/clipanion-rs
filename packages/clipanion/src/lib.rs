pub use clipanion_core as core;

extern crate clipanion_derive;

pub mod cli {
    pub use clipanion_derive::command;
}

pub mod advanced;
pub mod details;

#[macro_export]
macro_rules! new {
    ($($command:ty),+ $(,)?) => { {
        struct CommandSet {};

        impl clipanion::details::CommandSet for CommandSet {
            fn register_to_cli_builder(builder: &mut clipanion::core::CliBuilder) -> Result<(), clipanion::core::BuildError> {
                use clipanion::details::CommandController;

                $(<$command>::compile_cli_to_state_machine(builder.add_command())?;)*

                Ok(())
            }

            fn execute_cli_state(state: clipanion::core::RunState) -> std::process::ExitCode {
                use clipanion::details::CommandController;
                use clipanion::details::ToExitCode;

                // We can't use recursive macros to generate match arms, and
                // it's not possible yet to get the index of the current
                // iteration in a `for` loop, so we have to use a manual
                // counter here and pray the compiler optimizes it.
                let mut index = state.selected_index.unwrap();

                $({
                    if index == 0 {
                        let mut command = <$command>::default();
                        command.hydrate_cli_from_state(state);
                        return command.execute().to_exit_code();
                    } else {
                        index -= 1;
                    }
                }),*

                std::unreachable!();
            }
        };

        clipanion::advanced::Cli::<CommandSet>::new()
    } };
}
