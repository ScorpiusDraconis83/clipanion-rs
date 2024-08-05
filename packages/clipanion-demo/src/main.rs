use clipanion::command;

#[command]
#[derive(Debug)]
#[cli::path("cp")]
struct CpCommand {
    #[cli::option("-r,--recursive")]
    recursive: bool,

    #[cli::positional]
    sources: Vec<String>,

    #[cli::positional]
    destination: String,
}

impl CpCommand {
    pub fn execute(&self) {
        println!("{:?}", self);
    }
}

fn main() {
    clipanion::new![CpCommand].run_default();
}
