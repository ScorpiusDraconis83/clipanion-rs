use crate::details::CommandSet;

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

        T::register_to_cli_builder(&mut cli.builder)
            .unwrap();

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

        T::execute_cli_state(state)
    }

    pub fn run_default(&self) -> std::process::ExitCode {
        self.run(Default::default())
    }
}
