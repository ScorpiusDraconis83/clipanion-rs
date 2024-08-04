pub use clipanion_core as core;

extern crate clipanion_derive;
pub use clipanion_derive::command;

pub mod advanced;

#[macro_export]
macro_rules! new {
    ($($command:ty),+ $(,)?) => { {
        struct CommandSet {};

        impl clipanion::advanced::CommandSet for CommandSet {
            fn attach(builder: &mut clipanion::core::CliBuilder) {
                use clipanion::advanced::CommandController;

                $(<$command>::compile(builder.add_command());)*
            }

            fn execute(state: clipanion::core::RunState) -> std::process::ExitCode {
                use clipanion::advanced::CommandController;
                use clipanion::advanced::ToExitCode;

                let mut index = state.selected_index.unwrap();

                $({
                    if index == 0 {
                        let mut command = <$command>::default();
                        command.hydrate_cli_from(state);
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
