use clipanion::cli;

#[derive(Debug)]
#[cli::command]
#[cli::path("cp")]
struct Cp {
    #[cli::option("-r,--recursive", help = "Copy directories recursively", initial = false)]
    recursive: bool,

    sources: Vec<String>,
    destination: String,
}

impl Cp {
    pub fn execute(&self) {
        println!("{:?}", self);
    }
}

fn main() {
    clipanion::new![Cp].run_default();
}
