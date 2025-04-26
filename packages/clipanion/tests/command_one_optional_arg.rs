use clipanion::{advanced::Environment, details::CommandProvider, prelude::{Cli, *}, program};

#[cli::command(default)]
struct MyCommand {
    value: Option<String>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

#[test]
fn it_works() {
    let cli = MyCli::build_cli().unwrap();
    let env = Environment::default().with_argv(vec!["foo".to_string()]);

    let result
        = MyCli::parse_args(&cli, &env).unwrap();

    let MyCli::MyCommand(command) = result else {
        panic!("expected MyCommand");
    };

    assert_eq!(command.value, Some("foo".to_string()));
}
