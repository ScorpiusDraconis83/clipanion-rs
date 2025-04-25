#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Arg<'a> {
    StartOfInput,
    User(&'a str),
    EndOfInput,
    EndOfPartialInput,
}

impl<'a> Arg<'a> {
    pub fn starts_with(&self, s: &Arg) -> bool {
        match (self, s) {
            (Arg::User(a), Arg::User(b)) => a.starts_with(b),
            _ => false,
        }
    }

    pub fn unwrap_user(&self) -> &str {
        match self {
            Arg::User(s) => s,
            _ => panic!("Expected user argument"),
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
