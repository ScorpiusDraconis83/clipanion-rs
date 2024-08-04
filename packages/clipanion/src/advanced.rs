/**
 * Used to define the properties of the CLI. In general you can ignore this and
 * just use the `run_with_default()` function instead.
 */
pub struct Info {
    pub argv: Vec<String>,
    pub name: String,
    pub version: String,
    pub about: String,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            argv: std::env::args().skip(1).collect(),
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            about: env!("CARGO_PKG_DESCRIPTION").to_string(),
        }
    }
}

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
    fn hydrate_cli_from(&mut self, state: clipanion_core::RunState);
    fn compile(builder: &mut clipanion_core::CommandBuilder) -> Result<clipanion_core::Machine, clipanion_core::BuildError>;
}

/**
 * Internal trait implemented by the `new!` macro. Used to statically aggregate
 * multiple commands together.
 */
pub trait CommandSet {
    fn attach(builder: &mut clipanion_core::CliBuilder);
    fn execute(state: clipanion_core::RunState) -> std::process::ExitCode;
}

pub struct Cli<T> {
    builder: clipanion_core::CliBuilder,
    phantom: std::marker::PhantomData<T>,
}

impl<T: CommandSet> Cli<T> {
    pub fn new() -> Self {
        let mut cli = Self {
            builder: clipanion_core::CliBuilder::new(),
            phantom: Default::default(),
        };

        T::attach(&mut cli.builder);

        cli
    }

    pub fn run(&self, opts: Info) -> std::process::ExitCode {
        let machine = self.builder.compile();

        let state
            = clipanion_core::run_machine(&machine, &opts.argv).unwrap();

        if state.selected_index == Some(-1) {
            println!("TODO: Show the help message");
            return std::process::ExitCode::from(1);
        }

        T::execute(state)
    }

    pub fn run_default(&self) -> std::process::ExitCode {
        self.run(Default::default())
    }
}
