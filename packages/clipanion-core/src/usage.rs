use colored::Colorize;

use crate::{CommandSpec, Info};

pub struct CommandUsageOptions {
    pub detailed: bool,
    pub inline_options: bool,
}

pub struct CommandUsageResult {
    pub command_spec: CommandSpec,
}

impl CommandUsageResult {
    pub fn new(command_spec: CommandSpec) -> Self {
        Self {
            command_spec,
        }
    }

    pub fn oneliner(&self, info: &Info) -> String {
        let usage_line
            = format!("› {} {}", info.binary_name, self.command_spec);

        let usage_line = match info.colorized {
            true => usage_line.bright_white().to_string(),
            false => usage_line.to_string(),
        };

        usage_line
    }
}

pub struct OptionUsage {
    pub preferred_name: String,
    pub name_set: Vec<String>,
    pub definition: String,
    pub description: String,
    pub required: bool,
}
