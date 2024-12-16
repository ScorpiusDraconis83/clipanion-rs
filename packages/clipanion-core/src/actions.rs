use crate::{machine::MachineContext, runner::{OptionValue, Positional, RunState, Token}, shared::{Arg, HELP_COMMAND_INDEX}, CommandError, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum Reducer {
    #[default]
    None,
    InhibateOptions,
    PushBatch,
    PushBound,
    PushOptional,
    PushFalse(String),
    PushNone(String),
    PushPath,
    PushPositional,
    PushRest,
    AppendStringValue,
    InitializeState(usize, Vec<String>),
    PushTrue(String),
    SetError(CommandError),
    SetOptionArityError,
    AcceptState,
    ResetStringValue,
    UseHelp,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    Always,
    IsBatchOption,
    IsBoundOption,
    IsExactOption(String),
    IsNegatedOption(String),
    IsHelp,
    IsNotOptionLike,
    IsOptionLike,
    IsUnsupportedOption,
    IsInvalidOption,
}

pub fn apply_reducer(reducer: &Reducer, context: &MachineContext, state: &RunState, arg: &Arg, segment_index: usize) -> RunState {
    match reducer {
        Reducer::InhibateOptions => {
            let mut state = state.clone();
            state.ignore_options = true;
            state
        }

        Reducer::PushBatch => {
            let arg = arg.unwrap_user();
            let mut state = state.clone();

            for t in 1..arg.len() {
                let name = format!("-{}", &arg[t..t + 1]);

                let preferred_name
                    = context.preferred_names.get(&name).unwrap();

                let slice = match t == 1 {
                    true => (0, 2),
                    false => (t, t + 1),
                }; 

                state.options.push((
                    preferred_name.clone(),
                    OptionValue::Bool(true),
                ));

                state.tokens.push(Token::Option {
                    segment_index,
                    slice: Some(slice),
                    option: preferred_name.clone(),
                });
            }

            state
        }

        Reducer::PushBound => {
            let arg = arg.unwrap_user();
            let mut state = state.clone();

            let (name, value) = arg.split_at(arg.find('=').unwrap());

            state.options.push((
                name.to_string(),
                OptionValue::String(value[1..].to_string()),
            ));

            state.tokens.push(Token::Option {
                segment_index,
                slice: Some((0, name.len())),
                option: name.to_string(),
            });

            state.tokens.push(Token::Assign {
                segment_index,
                slice: (name.len(), name.len() + 1),
            });

            state.tokens.push(Token::Value {
                segment_index,
                slice: Some((name.len() + 1, value.len())),
            });

            state
        }

        Reducer::PushOptional => {
            let arg = arg.unwrap_user();
            let mut state = state.clone();
            state.positionals.push(Positional::Optional(arg.to_string()));
            state
        }

        Reducer::PushFalse(name) => {
            let mut state = state.clone();

            state.options.push((
                name.to_string(),
                OptionValue::Bool(false),
            ));

            state.tokens.push(Token::Option {
                segment_index,
                slice: None,
                option: name.to_string(),
            });

            state
        }

        Reducer::PushNone(name) => {
            let mut state = state.clone();

            state.options.push((
                name.to_string(),
                OptionValue::None,
            ));

            state.tokens.push(Token::Option {
                segment_index,
                slice: None,
                option: name.to_string(),
            });

            state
        }

        Reducer::PushPath => {
            let arg = arg.unwrap_user();
            let mut state = state.clone();
            state.path.push(arg.to_string());
            state
        }

        Reducer::PushPositional => {
            let arg = arg.unwrap_user();
            let mut state = state.clone();
            state.positionals.push(Positional::Required(arg.to_string()));
            state
        }

        Reducer::PushRest => {
            let arg = arg.unwrap_user();
            let mut state = state.clone();
            state.positionals.push(Positional::Rest(arg.to_string()));
            state
        }

        Reducer::AppendStringValue => {
            let arg = arg.unwrap_user();
            let mut state = state.clone();

            let last_option = state.options.last_mut().unwrap();

            match last_option.1 {
                OptionValue::None => {
                    last_option.1 = OptionValue::Array(vec![arg.to_string()]);
                }

                OptionValue::Array(ref mut values) => {
                    values.push(arg.to_string());
                }

                _ => {
                    panic!("Expected None or Array");
                }
            }

            state.tokens.push(Token::Value {
                segment_index,
                slice: None,
            });

            state
        }

        Reducer::PushTrue(name) => {
            let mut state = state.clone();

            state.options.push((
                name.to_string(),
                OptionValue::Bool(true),
            ));

            state.tokens.push(Token::Option {
                segment_index,
                slice: None,
                option: name.to_string(),
            });

            state
        }

        Reducer::SetError(error) => {
            let mut state = state.clone();
            state.error_message = Some(Error::CommandError(state.candidate_index, error.clone()));
            state
        }

        Reducer::SetOptionArityError => {
            let mut state = state.clone();
            state.error_message = Some(Error::CommandError(state.candidate_index, CommandError::MissingOptionArguments));
            state
        }

        Reducer::AcceptState => {
            let mut state = state.clone();
            state.selected_index = Some(state.candidate_index);
            state
        }

        Reducer::ResetStringValue => {
            let arg = arg.unwrap_user();
            let mut state = state.clone();

            let last_option = state.options.last_mut().unwrap();
            last_option.1 = OptionValue::String(arg.to_string());

            state.tokens.push(Token::Value {
                segment_index,
                slice: None,
            });

            state
        }

        Reducer::UseHelp => {
            let mut state = state.clone();
            state.selected_index = Some(HELP_COMMAND_INDEX);
            state.options = vec![("--command-index".to_string(), OptionValue::String(format!("{}", state.candidate_index)))];
            state.positionals.clear();
            state
        }

        Reducer::InitializeState(candidate_index, required_options) => {
            let mut state = state.clone();
            state.candidate_index = *candidate_index;
            state.required_options = required_options.clone();
            state
        }

        Reducer::None => {
            state.clone()
        }
    }
}

fn is_valid_option(option: &str) -> bool {
    if option.starts_with("--") {
        option.chars().skip(2).all(|c| c.is_alphanumeric() || c == '-')
    } else if option.starts_with("-") {
        option.chars().skip(1).all(|c| c.is_alphabetic())
    } else {
        false
    }
}

pub fn apply_check(check: &Check, context: &MachineContext, state: &RunState, arg: &Arg, _segment_index: usize) -> bool {
    match check {
        Check::Always => true,

        Check::IsBatchOption => {
            let arg = arg.unwrap_user();

            !state.ignore_options && arg.starts_with('-') && arg.len() > 2 && arg.chars().skip(1).all(|c| {
                c.is_ascii_alphanumeric() && context.preferred_names.contains_key(&format!("-{}", &c.to_string()))
            })
        }

        Check::IsBoundOption => {
            let arg = arg.unwrap_user();

            !state.ignore_options && arg.find('=').map_or(false, |i| {
                context.preferred_names.get(arg.split_at(i).0).map_or(false, |preferred_name| {
                    context.valid_bindings.contains(preferred_name)
                })
            })
        }

        Check::IsExactOption(needle) => {
            let arg = arg.unwrap_user();

            !state.ignore_options && arg == needle
        }

        Check::IsHelp => {
            let arg = arg.unwrap_user();

            !state.ignore_options && (arg == "--help" || arg == "-h" || arg.starts_with("--help="))
        }

        Check::IsNegatedOption(needle) => {
            let arg = arg.unwrap_user();

            !state.ignore_options && arg.starts_with("--no-") && arg[5..] == needle[2..]
        }

        Check::IsNotOptionLike => {
            let arg = arg.unwrap_user();

            state.ignore_options || arg == "-" || !arg.starts_with('-')
        }

        Check::IsOptionLike => {
            let arg = arg.unwrap_user();

            !state.ignore_options && arg != "-" && arg.starts_with('-')
        }

        Check::IsUnsupportedOption => {
            let arg = arg.unwrap_user();

            !state.ignore_options && arg.starts_with("-") && is_valid_option(arg) && !context.preferred_names.contains_key(arg)
        }

        Check::IsInvalidOption => {
            let arg = arg.unwrap_user();

            !state.ignore_options && arg.starts_with("-") && !is_valid_option(arg)
        }
    }
}
