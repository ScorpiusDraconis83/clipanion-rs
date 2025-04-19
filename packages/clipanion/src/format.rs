use std::{fmt::Display, marker::PhantomData};

use colored::Colorize;

use crate::{advanced::Info, details::CommandProvider};

fn format_usage_line(info: &Info, usage_line: &str) -> String {
    let usage_line = format!("$ {} {}", info.binary_name, usage_line);

    match info.colorized {
        true => usage_line.bright_white().to_string(),
        false => usage_line.to_string(),
    }
}

pub struct Formatter<S> {
    phantom: PhantomData<S>,
}

impl<S: CommandProvider> Formatter<S> {
    fn format_command_usages(info: &Info, command_indices: &[usize]) -> String {
        let mut result = String::new();

        let mut usage_lines = command_indices.iter()
            .map(|command_index| S::command_usage(*command_index, clipanion_core::CommandUsageOptions {detailed: false, inline_options: true}).unwrap().usage)
            .collect::<Vec<_>>();

        usage_lines.sort();

        for usage_line in usage_lines {
            result += "\n";
            result += &format_usage_line(info, &usage_line);
        }
    
        result
    }

    pub fn format_error<E: Display, I: AsRef<[usize]>>(info: &Info, err_type: &str, err: &E, command_indices: I) -> String {
        let mut result = String::new();
    
        result += &match info.colorized {
            true => format!("{}:", err_type).bright_red().to_string(),
            false => format!("{}:", err_type).to_string(),
        };
    
        result += " ";
        result += &err.to_string();
    
        if !command_indices.as_ref().is_empty() {
            result += "\n";
            result += &Self::format_command_usages(info, command_indices.as_ref());
        }
    
        result
    }

    pub fn format_parse_error(info: &Info, err: &clipanion_core::Error) -> String {
        match err {
            clipanion_core::Error::AmbiguousSyntax(candidate_indices)
                => Self::format_error(info, "Usage Error", &"The provided arguments are ambiguous and need to be refined further. Possible options are:", candidate_indices),
            clipanion_core::Error::CommandError(command_index, err)
                => Self::format_error(info, "Usage Error", err, [*command_index]),
            clipanion_core::Error::InternalError
                => Self::format_error(info, "Usage Error", &"An internal error occurred.", []),
            clipanion_core::Error::NotFound(suggested_indices)
                => Self::format_error(info, "Usage Error", &"The specified command was not found. Did you mean one of those commands?", suggested_indices),
        }
    }
}

