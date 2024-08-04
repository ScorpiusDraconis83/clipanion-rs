use clipanion::command;

#[command]
#[cli::path("add", "foo")]
struct AddCommand {
    #[cli::option("-P,--peer")]
    cli: bool,
}

impl AddCommand {
    pub fn execute(&self) {
        println!("AddCommand::execute()");
    }
}

fn main() {
    clipanion::new![AddCommand].run_default();
}
