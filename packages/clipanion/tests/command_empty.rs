use clipanion::{advanced::Environment, details::CommandProvider, prelude::{Cli, *}, program};

#[cli::command(default)]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

#[test]
fn it_works() {
    let cli = MyCli::build_cli().unwrap();
    let env = Environment::default().with_argv(vec![]);

    let result
        = MyCli::parse_args(&cli, &env).unwrap();

    let MyCli::MyCommand(_) = result else {
        panic!("expected MyCommand");
    };
}
