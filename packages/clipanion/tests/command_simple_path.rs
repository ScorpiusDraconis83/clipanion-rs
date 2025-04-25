use clipanion::{advanced::Environment, prelude::{Cli, *}, program};

#[cli::command]
#[cli::path("foo")]
#[derive(Clone)]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
    }
}

program!(MyCli, [
    MyCommand,
]);

#[test]
fn it_works() {
    let MyCli::MyCommand(_) = MyCli::parse_args(["foo"]).unwrap() else {
        panic!("failed to parse");
    };
}
