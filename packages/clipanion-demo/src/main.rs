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

/// Record changes to the repository.
#[cli::command]
#[cli::path("commit")]
struct GitCommitCommand {
    /// Automatically stage files that have been modified and deleted.
    #[cli::option("-a,--all", default = false)]
    all: bool,

    /// Allow empty commits.
    #[cli::option("--allow-empty", default = false)]
    allow_empty: bool,

    /// Replace the tip of the current branch by creating a new commit.
    #[cli::option("--amend", default = false)]
    amend: bool,

    /// Use the given message as the commit message.
    #[cli::option("-m,--message")]
    message: Option<String>,

    /// Commit the contents of the files that match the pathspec without recording the changes already added to the index.
    paths: Vec<String>,
}

impl GitCommitCommand {
    async fn execute(&self) {
    }
}

/// Remove files from the working tree and from the index.
#[cli::command]
#[cli::path("rm")]
struct GitRmCommand {
    /// Override the up-to-date check.
    #[cli::option("-f,--force", default = false)]
    force: bool,

    /// Unstage and remove paths only from the index.
    #[cli::option("--cached", default = false)]
    cached: bool,

    /// Don’t actually remove the file(s), just show if they exist and/or will be ignored.
    #[cli::option("-n,--dry-run", default = false)]
    dry_run: bool,

    /// Files to remove.
    paths: Vec<String>,
}

impl GitRmCommand {
    async fn execute(&self) {
    }
}

/// List the configuration variables in the config file.
#[cli::command]
#[cli::path("config", "list")]
struct GitConfigListCommand {
}

impl GitConfigListCommand {
    async fn execute(&self) {
    }
}

/// Retrieve the value of a configuration variable.
#[cli::command]
#[cli::path("config", "get")]
struct GitConfigGetCommand {
    /// The name of the configuration variable.
    name: String,
}

impl GitConfigGetCommand {
    async fn execute(&self) {
    }
}

/// Set the value of a configuration variable.
#[cli::command]
#[cli::path("config", "set")]
struct GitConfigSetCommand {
    /// The name of the configuration variable.
    name: String,

    /// The value of the configuration variable.
    value: String,
}

impl GitConfigSetCommand {
    async fn execute(&self) {
    }
}

#[cli::program(async)]
enum MyCli {
    GitAdd(GitAddCommand),
    GitCommit(GitCommitCommand),
    GitConfigGet(GitConfigGetCommand),
    GitConfigList(GitConfigListCommand),
    GitConfigSet(GitConfigSetCommand),
    GitRm(GitRmCommand),
}

#[tokio::main()]
async fn main() -> ExitCode {
    MyCli::run_default().await
}
