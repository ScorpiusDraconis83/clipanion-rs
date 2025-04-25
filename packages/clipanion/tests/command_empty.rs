use clipanion::{advanced::Environment, prelude::{Cli, *}, program};

static mut X: bool = false;

#[cli::command(default)]
struct MyCommand {}

impl MyCommand {
    fn execute(&self) {
        unsafe {
            X = true;
        }
    }
}

program!(MyCli, [MyCommand]);

#[test]
fn it_works() {
    MyCli::run(Environment::default().with_argv(vec![]));
    assert!(unsafe { X });
}
