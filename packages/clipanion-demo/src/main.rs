use std::process::ExitCode;

use clipanion::prelude::*;

/// Add file contents to the index.
#[cli::command]
#[cli::path("add")]
#[cli::example(command = "git add", description = "Add all files to the index.")]
struct GitAddCommand {
    /// Be verbose.
    #[cli::option("-v,--verbose", default = false)]
    verbose: bool,

    /// Don’t actually add the file(s), just show if they exist and/or will be ignored.
    #[cli::option("-d,--dry-run", default = false)]
    dry_run: bool,

    /// Allow adding otherwise ignored files.
    #[cli::option("-f,--force", default = false)]
    force: bool,

    /// Allow updating index entries outside of the sparse-checkout cone.
    #[cli::option("--sparse", default = false)]
    sparse: bool,

    /// Files to add content from.
    paths: Vec<String>,
}

impl GitAddCommand {
    async fn execute(&self) {
    }
}

#[cli::command]
#[cli::path("commit")]
#[cli::description("Record changes to the repository.")]
struct GitCommitCommand {
    #[cli::option("-a,--all", default = false, description = "Automatically stage files that have been modified and deleted.")]
    all: bool,

    #[cli::option("--allow-empty", default = false, description = "Allow empty commits.")]
    allow_empty: bool,

    #[cli::option("--amend", default = false, description = "Replace the tip of the current branch by creating a new commit.")]
    amend: bool,

    #[cli::option("-m,--message", description = "Use the given message as the commit message.")]
    message: Option<String>,

    #[cli::positional(description = "Commit the contents of the files that match the pathspec without recording the changes already added to the index.")]
    paths: Vec<String>,
}

impl GitCommitCommand {
    async fn execute(&self) {
    }
}

#[cli::command]
#[cli::path("rm")]
#[cli::description("Remove files from the working tree and from the index.")]
struct GitRmCommand {
    #[cli::option("-f,--force", default = false, description = "Override the up-to-date check.")]
    force: bool,

    #[cli::option("--cached", default = false, description = "Unstage and remove paths only from the index.")]
    cached: bool,

    #[cli::option("-n,--dry-run", default = false, description = "Don’t actually remove the file(s), just show if they exist and/or will be ignored.")]
    dry_run: bool,

    #[cli::positional(description = "Files to remove.")]
    paths: Vec<String>,
}

impl GitRmCommand {
    async fn execute(&self) {
    }
}

#[cli::program(async)]
enum MyCli {
    GitAdd(GitAddCommand),
    GitCommit(GitCommitCommand),
    GitRm(GitRmCommand),
}

#[tokio::main()]
async fn main() -> ExitCode {
    MyCli::run_default().await
}
