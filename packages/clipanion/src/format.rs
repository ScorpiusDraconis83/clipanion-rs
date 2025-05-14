use std::{fmt::Display, marker::PhantomData};

use clipanion_core::{CommandSpec, Info};
use colored::Colorize;

use crate::details::CommandProvider;

use std::collections::HashMap;
use std::env;
use std::fmt::Write;

pub fn format_fading_title_line(title: &str, total_length: usize, fade_len: usize) -> String {
    // Background target map (RGB)
    let target_colors: HashMap<&'static str, (u8, u8, u8)> = [
        ("ghostty", (39, 45, 52)),
        ("xterm",   (0, 0, 0)),
    ].iter().cloned().collect();

    let term_program = env::var("TERM_PROGRAM").unwrap_or_default().to_lowercase();
    let (r_target, g_target, b_target) = target_colors
        .get(term_program.as_str())
        .copied()
        .unwrap_or((0, 0, 0)); // Default: black

    // Decorative left part
    let left = "━━━ ";
    let title_str = format!("{}{} ", left, title);
    let visible_len = title_str.chars().count();

    // Calculate how many ━ are needed on the right
    let remaining = total_length.saturating_sub(visible_len);
    let fade_len = fade_len.min(remaining);
    let solid_len = remaining.saturating_sub(fade_len);

    let mut output = String::new();
    output.push_str("\x1b[1m"); // Bold
    output.push_str(&title_str);

    // Solid ━ characters before the fade starts
    for _ in 0..solid_len {
        output.push('━');
    }

    // Fading right side (from white to target background color)
    for i in 0..fade_len {
        let t = i as f32 / fade_len as f32;
        let r = ((1.0 - t) * 255.0 + t * r_target as f32) as u8;
        let g = ((1.0 - t) * 255.0 + t * g_target as f32) as u8;
        let b = ((1.0 - t) * 255.0 + t * b_target as f32) as u8;

        let _ = write!(
            output,
            "\x1b[38;2;{r};{g};{b}m━"
        );
    }

    output.push_str("\x1b[0m"); // Reset
    output
}

pub struct Formatter<S> {
    phantom: PhantomData<S>,
}

impl<S: CommandProvider> Formatter<S> {
    pub fn format_error<'cmds>(info: &Info, err_type: &str, err: &impl Display, command_specs: impl IntoIterator<Item = &'cmds CommandSpec>) -> String {
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
            result += "\n\n";
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

