use std::{fmt::Display, marker::PhantomData};

use clipanion_core::{CommandSpec, Info};
use colored::Colorize;

use crate::details::CommandProvider;

pub struct Formatter<S> {
    phantom: PhantomData<S>,
}

impl<S: CommandProvider> Formatter<S> {
    pub fn format_error<'a>(info: &Info, err_type: &str, err: &impl Display, command_specs: impl IntoIterator<Item = &'a CommandSpec>) -> String {
        let mut result = String::new();
    
        result += &match info.colorized {
            true => format!("{}:", err_type).bright_red().to_string(),
            false => format!("{}:", err_type).to_string(),
        };
    
        result += " ";
        result += &err.to_string();

        let usage_lines = command_specs.into_iter()
            .map(|command_spec| command_spec.usage().oneliner(info))
            .collect::<Vec<_>>();

        if !usage_lines.is_empty() {
            result += "\n";
            result += &usage_lines.join("\n");
        }
    
        result
    }

    pub fn format_parse_error(info: &Info, err: &clipanion_core::Error) -> String {
        match err {
            clipanion_core::Error::AmbiguousSyntax(candidate_specs)
                => Self::format_error(info, "Usage Error", &"The provided arguments are ambiguous and need to be refined further. Possible options are:", candidate_specs.iter().cloned()),

            clipanion_core::Error::CommandError(command_spec, err)
                => Self::format_error(info, "Usage Error", err, [*command_spec]),

            clipanion_core::Error::InternalError
                => Self::format_error(info, "Usage Error", &"An internal error occurred.", []),

            clipanion_core::Error::NotFound(suggested_specs)
                => Self::format_error(info, "Usage Error", &"The specified command was not found. Did you mean one of those commands?", suggested_specs.iter().cloned()),
        }
    }
}

