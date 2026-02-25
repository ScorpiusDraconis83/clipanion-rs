use std::collections::BTreeSet;

use crate::{
    builder::{Check, CommandSpec, Component, OptionSpec, Reducer, State},
    runner::{Runner, RunnerState},
    shared::{Arg, ArgKey, ERROR_NODE_ID},
    Machine,
};

/// A single completion candidate
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Completion {
    /// The completion text
    pub text: String,

    /// Description of this completion
    pub description: Option<String>,

    /// Whether this completion represents a path/command keyword
    pub is_path: bool,

    /// Whether this completion represents an option
    pub is_option: bool,
}

impl Completion {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            description: None,
            is_path: false,
            is_option: false,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn as_path(mut self) -> Self {
        self.is_path = true;
        self
    }

    pub fn as_option(mut self) -> Self {
        self.is_option = true;
        self
    }
}

/// Context for completion computation
#[derive(Debug, Clone)]
pub struct CompletionContext<'a> {
    /// Arguments before the current token
    pub args_before: Vec<&'a str>,

    /// The partial token being completed (empty if cursor is between tokens)
    pub current: &'a str,

    /// Arguments after the current token
    pub args_after: Vec<&'a str>,

    /// Position of cursor within the current token (0 = beginning)
    pub cursor_position: usize,
}

impl<'a> CompletionContext<'a> {
    pub fn new(args_before: Vec<&'a str>, current: &'a str, args_after: Vec<&'a str>, cursor_position: usize) -> Self {
        Self {
            args_before,
            current,
            args_after,
            cursor_position,
        }
    }

    /// Create a context from a full command line where the cursor is at the end
    pub fn from_args_at_end(args: Vec<&'a str>) -> Self {
        if args.is_empty() {
            Self::new(vec![], "", vec![], 0)
        } else {
            let last_arg = args[args.len() - 1];
            Self::new(args[..args.len() - 1].to_vec(), last_arg, vec![], last_arg.len())
        }
    }
}

/// Result of completion computation
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CompletionResult {
    pub completions: Vec<Completion>,
}

type CliMachine<'cmds> = Machine<'cmds, Option<Check<'cmds>>, Option<Reducer>>;

/// Compute completions for a CLI given the current context
pub fn compute_completions<'cmds, 'args>(
    commands: &[&'cmds CommandSpec],
    machine: &CliMachine<'cmds>,
    context: &CompletionContext<'args>,
) -> CompletionResult {
    // Run the machine with the arguments before the current token to get the current states
    let states = run_machine_partial(machine, &context.args_before);

    // Now analyze each state to find valid completions
    let mut completions = BTreeSet::new();

    for state in &states {
        if state.node_id == ERROR_NODE_ID {
            continue;
        }

        let node = &machine.nodes[state.node_id];
        let command = commands.get(state.context_id);

        // Build a set of already-used option component IDs
        let used_option_ids: std::collections::HashSet<usize> = state
            .option_values
            .iter()
            .map(|(id, _)| *id)
            .collect();

        // Collect static transitions (command paths, keywords)
        for (key, _transitions) in &node.statics {
            if let ArgKey::User(keyword) = key {
                // Check if the current partial matches
                if keyword.starts_with(context.current) {
                    completions.insert(Completion::new(*keyword).as_path());
                }
            }
        }

        // Collect dynamic transitions (options)
        for (check, _transition) in &node.dynamics {
            if let Some(Check::IsOption(name)) = check {
                // Don't suggest -- or -h/--help which are special
                if *name == "--" || *name == "-h" || *name == "--help" {
                    continue;
                }

                // Check if the current partial matches
                if name.starts_with(context.current) {
                    // Find the option spec to check if it's already used
                    if let Some(cmd) = command {
                        if let Some((component_id, opt)) = find_option_with_id_by_name(cmd, name) {
                            // Skip options that are already used and don't accept multiple values
                            let is_single_use = opt.min_len == 0 && opt.extra_len == Some(0);
                            if is_single_use && used_option_ids.contains(&component_id) {
                                continue;
                            }

                            let description = opt.documentation.as_ref()
                                .map(|doc| doc.description.clone());

                            let mut completion = Completion::new(*name).as_option();
                            if let Some(desc) = description {
                                completion = completion.with_description(desc);
                            }
                            completions.insert(completion);
                        }
                    } else {
                        // No command context, just add the option
                        completions.insert(Completion::new(*name).as_option());
                    }
                }
            }
        }

        // If current starts with -, filter to only options
        if !context.current.is_empty() && context.current.starts_with("-") && !state.post_double_dash {
            // Only show options
            completions.retain(|c| c.is_option);
        }
    }

    CompletionResult {
        completions: completions.into_iter().collect(),
    }
}

fn find_option_with_id_by_name<'cmds>(command: &'cmds CommandSpec, name: &str) -> Option<(usize, &'cmds OptionSpec)> {
    command.components.iter().enumerate().find_map(|(id, component)| {
        if let Component::Option(opt) = component {
            if opt.all_names().any(|n| n == name) {
                return Some((id, opt));
            }
        }
        None
    })
}

fn run_machine_partial<'cmds, 'args>(
    machine: &CliMachine<'cmds>,
    args: &[&'args str],
) -> Vec<State<'args>> {
    fn on_error<'args>(mut state: State<'args>, _: Arg<'args>) -> State<'args> {
        state.set_node_id(ERROR_NODE_ID);
        state
    }

    Runner::run_partial(machine, on_error, args)
}

/// Generate shell completion script
pub fn generate_completion_script(shell: Shell, command: &str) -> String {
    match shell {
        Shell::Bash => generate_bash_script(command),
        Shell::Zsh => generate_zsh_script(command),
        Shell::Fish => generate_fish_script(command),
    }
}

/// Supported shells for completion scripts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            _ => None,
        }
    }

    pub fn detect() -> Option<Self> {
        std::env::var("SHELL")
            .ok()
            .and_then(|s| {
                let shell_name = s.rsplit('/').next()?;
                Self::from_str(shell_name)
            })
    }
}

fn generate_bash_script(command: &str) -> String {
    // Extract the binary name from the command (last path component, no arguments)
    let binary_name = command
        .split_whitespace()
        .next()
        .unwrap_or(command)
        .rsplit('/')
        .next()
        .unwrap_or(command);

    format!(
        r#"# Bash completion script for {binary_name}
# Generated by clipanion-rs
# Add this to your ~/.bashrc or source it directly

_{binary_name}_completions() {{
    local cur prev words cword
    _init_completion || return

    local IFS=$'\n'
    local completions
    # cword is 1-based and includes the command name, so subtract 1 for 0-based index without command
    local index=$((cword - 1))
    completions=$({command} --clipanion-complete "$index" -- "${{words[@]:1}}" 2>/dev/null)

    if [[ -n "$completions" ]]; then
        COMPREPLY=($(compgen -W "$completions" -- "$cur"))
    fi
}}

complete -F _{binary_name}_completions {binary_name}
"#,
        binary_name = binary_name,
        command = command,
    )
}

fn generate_zsh_script(command: &str) -> String {
    let binary_name = command
        .split_whitespace()
        .next()
        .unwrap_or(command)
        .rsplit('/')
        .next()
        .unwrap_or(command);

    // Replace hyphens with underscores for valid zsh function name
    let func_name = binary_name.replace('-', "_");

    format!(
        r#"# Zsh completion script for {binary_name}
# Generated by clipanion-rs
# Add this to your ~/.zshrc or place in a file in your $fpath

_{func_name}() {{
    local -a completions
    local line

    # CURRENT is 1-based and includes command name, convert to 0-based index without command
    local index=$((CURRENT - 2))

    # Get completions from the CLI
    while IFS= read -r line; do
        completions+=("$line")
    done < <({command} --clipanion-complete "$index" -- "${{words[@]:1}}" 2>/dev/null)

    # Add completions
    if (( $#completions )); then
        compadd -a completions
    fi
}}

compdef _{func_name} {binary_name}
"#,
        binary_name = binary_name,
        func_name = func_name,
        command = command,
    )
}

fn generate_fish_script(command: &str) -> String {
    let binary_name = command
        .split_whitespace()
        .next()
        .unwrap_or(command)
        .rsplit('/')
        .next()
        .unwrap_or(command);

    format!(
        r#"# Fish completion script for {binary_name}
# Generated by clipanion-rs
# Save this file to ~/.config/fish/completions/{binary_name}.fish

function __fish_{binary_name}_completions
    set -l tokens (commandline -opc)
    set -l current (commandline -ct)

    # Remove the command name itself (first token)
    set -e tokens[1]

    # Count tokens to get the index (current token position)
    set -l index (count $tokens)

    # Add the current token to the args if it's not empty
    if test -n "$current"
        set tokens $tokens "$current"
    end

    {command} --clipanion-complete "$index" -- $tokens 2>/dev/null
end

complete -c {binary_name} -f -a '(__fish_{binary_name}_completions)'
"#,
        binary_name = binary_name,
        command = command,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::CliBuilder, PositionalSpec};

    fn create_simple_cli() -> Vec<CommandSpec> {
        vec![
            CommandSpec {
                primary_path: vec!["add".to_string()],
                components: vec![
                    Component::Option(OptionSpec::boolean("-v,--verbose")),
                    Component::Option(OptionSpec::parametrized("-m,--message")),
                    Component::Positional(PositionalSpec::rest()),
                ],
                ..Default::default()
            },
            CommandSpec {
                primary_path: vec!["commit".to_string()],
                components: vec![
                    Component::Option(OptionSpec::boolean("-a,--all")),
                    Component::Option(OptionSpec::parametrized("-m,--message")),
                ],
                ..Default::default()
            },
            CommandSpec {
                primary_path: vec!["checkout".to_string()],
                components: vec![
                    Component::Option(OptionSpec::boolean("-b")),
                    Component::Positional(PositionalSpec::optional()),
                ],
                ..Default::default()
            },
        ]
    }

    #[test]
    fn test_complete_empty_input() {
        let specs = create_simple_cli();
        let mut builder = CliBuilder::new();
        for spec in &specs {
            builder.add_command(spec);
        }
        let machine = builder.compile();

        let command_refs: Vec<&CommandSpec> = specs.iter().collect();
        let context = CompletionContext::new(vec![], "", vec![], 0);
        let result = compute_completions(&command_refs, &machine, &context);

        // Should suggest command paths
        let texts: Vec<&str> = result.completions.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"add"));
        assert!(texts.contains(&"commit"));
        assert!(texts.contains(&"checkout"));
    }

    #[test]
    fn test_complete_partial_command() {
        let specs = create_simple_cli();
        let mut builder = CliBuilder::new();
        for spec in &specs {
            builder.add_command(spec);
        }
        let machine = builder.compile();

        let command_refs: Vec<&CommandSpec> = specs.iter().collect();
        let context = CompletionContext::new(vec![], "co", vec![], 2);
        let result = compute_completions(&command_refs, &machine, &context);

        // Should only suggest commands starting with "co" (commit, not checkout which starts with "ch")
        let texts: Vec<&str> = result.completions.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"commit"));
        assert!(!texts.contains(&"checkout")); // checkout starts with "ch", not "co"
        assert!(!texts.contains(&"add"));
    }

    #[test]
    fn test_complete_options_after_command() {
        let specs = create_simple_cli();
        let mut builder = CliBuilder::new();
        for spec in &specs {
            builder.add_command(spec);
        }
        let machine = builder.compile();

        let command_refs: Vec<&CommandSpec> = specs.iter().collect();
        let context = CompletionContext::new(vec!["add"], "-", vec![], 1);
        let result = compute_completions(&command_refs, &machine, &context);

        // Should suggest options for 'add' command
        let texts: Vec<&str> = result.completions.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"-v"));
        assert!(texts.contains(&"--verbose"));
        assert!(texts.contains(&"-m"));
        assert!(texts.contains(&"--message"));
    }

    #[test]
    fn test_complete_long_option_prefix() {
        let specs = create_simple_cli();
        let mut builder = CliBuilder::new();
        for spec in &specs {
            builder.add_command(spec);
        }
        let machine = builder.compile();

        let command_refs: Vec<&CommandSpec> = specs.iter().collect();
        let context = CompletionContext::new(vec!["add"], "--v", vec![], 3);
        let result = compute_completions(&command_refs, &machine, &context);

        // Should only suggest --verbose
        let texts: Vec<&str> = result.completions.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"--verbose"));
        assert!(!texts.contains(&"--message"));
    }

    #[test]
    fn test_complete_from_args_at_end() {
        let specs = create_simple_cli();
        let mut builder = CliBuilder::new();
        for spec in &specs {
            builder.add_command(spec);
        }
        let machine = builder.compile();

        let command_refs: Vec<&CommandSpec> = specs.iter().collect();
        let context = CompletionContext::from_args_at_end(vec!["add", "--"]);
        let result = compute_completions(&command_refs, &machine, &context);

        // Should suggest long options
        let texts: Vec<&str> = result.completions.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"--verbose"));
        assert!(texts.contains(&"--message"));
    }

    #[test]
    fn test_filter_already_used_options() {
        let specs = create_simple_cli();
        let mut builder = CliBuilder::new();
        for spec in &specs {
            builder.add_command(spec);
        }
        let machine = builder.compile();

        let command_refs: Vec<&CommandSpec> = specs.iter().collect();

        // After using --verbose, it should not be suggested again
        let context = CompletionContext::new(vec!["add", "--verbose"], "-", vec![], 1);
        let result = compute_completions(&command_refs, &machine, &context);

        let texts: Vec<&str> = result.completions.iter().map(|c| c.text.as_str()).collect();
        assert!(!texts.contains(&"--verbose")); // Should be filtered out
        assert!(!texts.contains(&"-v")); // Alias should also be filtered out
        assert!(texts.contains(&"-m")); // Other options should still be available
        assert!(texts.contains(&"--message"));
    }

    #[test]
    fn test_shell_detection() {
        assert_eq!(Shell::from_str("bash"), Some(Shell::Bash));
        assert_eq!(Shell::from_str("BASH"), Some(Shell::Bash));
        assert_eq!(Shell::from_str("zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::from_str("fish"), Some(Shell::Fish));
        assert_eq!(Shell::from_str("unknown"), None);
    }

    #[test]
    fn test_bash_script_generation() {
        let script = generate_completion_script(Shell::Bash, "my-cli");
        assert!(script.contains("complete -F _my-cli_completions my-cli"));
        assert!(script.contains("--clipanion-complete"));
        assert!(script.contains("-- \"${words[@]:1}\"")); // Uses -- separator
    }

    #[test]
    fn test_zsh_script_generation() {
        let script = generate_completion_script(Shell::Zsh, "my-cli");
        assert!(script.contains("compdef _my_cli my-cli")); // Function name has underscores
        assert!(script.contains("--clipanion-complete"));
        assert!(script.contains("-- \"${words[@]:1}\"")); // Uses -- separator
        assert!(script.contains("compadd -a completions")); // Uses compadd
    }

    #[test]
    fn test_fish_script_generation() {
        let script = generate_completion_script(Shell::Fish, "my-cli");
        assert!(script.contains("complete -c my-cli"));
        assert!(script.contains("--clipanion-complete"));
        assert!(script.contains("-- $tokens")); // Uses -- separator
    }

    #[test]
    fn test_script_with_path_command() {
        let script = generate_completion_script(Shell::Bash, "/usr/local/bin/my-cli");
        // Should extract binary name from path
        assert!(script.contains("complete -F _my-cli_completions my-cli"));
    }
}
