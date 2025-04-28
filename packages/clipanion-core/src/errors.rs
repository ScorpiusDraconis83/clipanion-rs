use crate::builder::CommandSpec;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    #[error("{0}")]
    Custom(String),

    #[error("Missing required option argument {0}")]
    MissingOptionArguments(String),

    #[error("Unsupported option name")]
    UnknownOption,

    #[error("Invalid option name")]
    InvalidOption,

    #[error("Missing required positional argument")]
    MissingPositionalArguments,

    #[error("Extraneous positional arguments")]
    ExtraneousPositionalArguments,
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        CommandError::Custom(message)
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error<'cmds> {
    #[error("The provided arguments are ambiguous and need to be refined further. Possible options are:")]
    AmbiguousSyntax(Vec<&'cmds CommandSpec>),

    #[error("{1}")]
    CommandError(&'cmds CommandSpec, CommandError),

    #[error("Something unexpected happened; this seems to be a bug in the CLI framework itself")]
    InternalError,

    #[error("The provided arguments don't match any known syntax; use `--help` to get a list of possible options")]
    NotFound(Vec<&'cmds CommandSpec>),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    #[error("Commands can only define a single rest parameter")]
    MultipleRestParameters,

    #[error("Commands aren't allowed to define optional parameters after a rest parameter")]
    OptionalParametersAfterRest,

    #[error("Commands aren't allowed to define optional parameters after trailing positionals")]
    OptionalParametersAfterTrailingPositionals,

    #[error("Commands aren't allowed to define rest parameters after trailing positionals")]
    RestAfterTrailingPositionals,

    #[error("TODO: I don't remember the details of this error right at this moment")]
    ArityTooHighForNonBindingOption,
}
