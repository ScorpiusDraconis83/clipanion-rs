/**
 * Internal trait used to convert whatever the `execute()` function returns
 * into an exit code. It makes it easier to return `()` from simple commands
 * without having to return a specific number.
 */
pub trait ToExitCode {
    fn to_exit_code(&self) -> std::process::ExitCode;
}

impl ToExitCode for () {
    fn to_exit_code(&self) -> std::process::ExitCode {
        std::process::ExitCode::from(0)
    }
}

impl ToExitCode for u8 {
    fn to_exit_code(&self) -> std::process::ExitCode {
        std::process::ExitCode::from(*self)
    }
}

/**
 * Internal trait implemented by the #[command] attribute.
 */
pub trait CommandController {
    fn hydrate_cli_from_state(&mut self, state: clipanion_core::RunState);
    fn compile_cli_to_state_machine(builder: &mut clipanion_core::CommandBuilder) -> Result<clipanion_core::Machine, clipanion_core::BuildError>;
}

/**
 * Internal trait implemented by the `new!` macro. Used to statically aggregate
 * multiple commands together in a single type.
 */
pub trait CommandSet {
    fn register_to_cli_builder(builder: &mut clipanion_core::CliBuilder) -> Result<(), clipanion_core::BuildError>;
    fn execute_cli_state(state: clipanion_core::RunState) -> std::process::ExitCode;
}
