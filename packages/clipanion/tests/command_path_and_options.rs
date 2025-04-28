use clipanion::{advanced::Environment, details::CommandProvider, prelude::*, program};

#[cli::command]
#[cli::path("foo", "bar")]
struct MyCommand {
    #[cli::option("--my-option")]
    my_option: Option<bool>,
}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [MyCommand]);

#[test]
fn it_works() {
    let cli = MyCli::build_cli().unwrap();
    let env = Environment::default().with_argv(vec!["foo".to_string(), "bar".to_string(), "--my-option".to_string()]);

    let result
        = MyCli::parse_args(&cli, &env).unwrap();

    let MyCli::MyCommand(command) = result else {
        panic!("expected MyCommand");
    };

    assert_eq!(command.my_option, Some(true));
}

#[test]
fn it_works_if_the_option_is_first() {
    let cli = MyCli::build_cli().unwrap();
    let env = Environment::default().with_argv(vec!["--my-option".to_string(), "foo".to_string(), "bar".to_string()]);

    let result
        = MyCli::parse_args(&cli, &env).unwrap();

    let MyCli::MyCommand(command) = result else {
        panic!("expected MyCommand");
    };

    assert_eq!(command.my_option, Some(true));
}

#[test]
fn it_works_if_the_option_is_between_the_path_segments() {
    let cli = MyCli::build_cli().unwrap();
    let env = Environment::default().with_argv(vec!["foo".to_string(), "--my-option".to_string(), "bar".to_string()]);

    let result
        = MyCli::parse_args(&cli, &env).unwrap();

    let MyCli::MyCommand(command) = result else {
        panic!("expected MyCommand");
    };

    assert_eq!(command.my_option, Some(true));
}
