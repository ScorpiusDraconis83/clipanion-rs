#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "tokens", derive(serde::Serialize))]
pub enum Arg<'a> {
    StartOfInput,
    User(&'a str, usize),
    EndOfInput,
    EndOfPartialInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArgKey<'a> {
    StartOfInput,
    User(&'a str),
    EndOfInput,
    EndOfPartialInput,
}

impl<'a> From<Arg<'a>> for ArgKey<'a> {
    fn from(arg: Arg<'a>) -> Self {
        match arg {
            Arg::StartOfInput => ArgKey::StartOfInput,
            Arg::User(s, _) => ArgKey::User(s),
            Arg::EndOfInput => ArgKey::EndOfInput,
            Arg::EndOfPartialInput => ArgKey::EndOfPartialInput,
        }
    }
}

pub const HELP_COMMAND_INDEX: usize = usize::MAX;

pub const INITIAL_NODE_ID: usize = 0;
pub const SUCCESS_NODE_ID: usize = 1;
pub const ERROR_NODE_ID: usize = 2;
pub const CUSTOM_NODE_ID: usize = 3;

pub fn is_terminal_node(id: usize) -> bool {
    id == SUCCESS_NODE_ID || id == ERROR_NODE_ID
}    
