use std::process::ExitCode;

use clipanion::prelude::*;

/// Add file contents to the index.
///
/// This command updates the index using the current content found in the working tree, to prepare the content staged for the next commit. It
/// typically adds the current content of existing paths as a whole, but with some options it can also be used to add content with only part of the
/// changes made to the working tree files applied, or remove paths that do not exist in the working tree anymore.
///
/// The "index" holds a snapshot of the content of the working tree, and it is this snapshot that is taken as the contents of the next commit. Thus
/// after making any changes to the working tree, and before running the commit command, you must use the add command to add any new or modified
/// files to the index.
///
/// This command can be performed multiple times before a commit. It only adds the content of the specified file(s) at the time the add command is
/// run; if you want subsequent changes included in the next commit, then you must run git add again to add the new content to the index.
///
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
///
/// Create a new commit containing the current contents of the index and the given log message describing the changes. The new commit is a direct
/// child of HEAD, usually the tip of the current branch, and the branch is updated to point to it (unless no branch is associated with the working
/// tree, in which case `HEAD` is "detached" as described in `git checkout`).
///
/// The content to be committed can be specified in several ways:
///
/// - by using `git add` to incrementally "add" changes to the index before using the commit command (Note: even modified files must be "added");
///
/// - by using `git rm` to remove files from the working tree and the index, again before using the commit command;
///
/// - by listing files as arguments to the `commit` command (without `--interactive` or `--patch` switch), in which case the commit will ignore
///   changes staged in the index, and instead record the current content of the listed files (which must already be known to Git);
///
/// - by using the `-a` switch with the `commit` command to automatically "add" changes from all known files (i.e. all files that are already listed
///   in the index) and to automatically "rm" files in the index that have been removed from the working tree, and then perform the actual commit;
///
/// - by using the `--interactive` or `--patch` switches with the `commit` command to decide one by one which files or hunks should be part of the
///   commit in addition to contents in the index, before finalizing the operation. See the “Interactive Mode” section of `git add` to learn how to
///   operate these modes.
///
/// The `--dry-run` option can be used to obtain a summary of what is included by any of the above for the next commit by giving the same set of
/// parameters (options and paths).
///
/// If you make a commit and then find a mistake immediately after that, you can recover from it with `git reset`.
///
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
///
/// Remove files matching pathspec from the index, or from the working tree and the index. `git rm` will not remove a file from just your working
/// directory. (There is no option to remove a file only from the working tree and yet keep it in the index; use `/bin/rm` if you want to do that.)
/// The files being removed have to be identical to the tip of the branch, and no updates to their contents can be staged in the index, though that
/// default behavior can be overridden with the `-f` option. When `--cached` is given, the staged content has to match either the tip of the branch
/// or the file on disk, allowing the file to be removed from just the index. When sparse-checkouts are in use (see `git sparse checkout`), `git rm`
/// will only remove paths within the sparse-checkout patterns.
///
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
