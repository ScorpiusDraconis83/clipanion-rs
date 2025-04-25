use crate::builder2::CommandSpec;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    #[error("The option `{0}` expects a value")]
    Custom(String),

    #[error("Missing required option argument")]
    MissingOptionArguments,

    #[error("Unsupported option name")]
    UnknownOption,

    #[error("Invalid option name")]
    InvalidOption,

    #[error("Missing required positional argument")]
    MissingPositionalArguments,

    #[error("Extraneous positional arguments")]
    ExtraneousPositionalArguments,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error<'a> {
    #[error("The provided arguments are ambiguous and need to be refined further. Possible options are:")]
    AmbiguousSyntax(Vec<&'a CommandSpec>),

    #[error("{1}")]
    CommandError(&'a CommandSpec, CommandError),

    #[error("Something unexpected happened; this seems to be a bug in the CLI framework itself")]
    InternalError,

    #[error("The provided arguments don't match any known syntax; use `--help` to get a list of possible options")]
    NotFound(Vec<&'a CommandSpec>),
}

#[derive(thiserror::Error, Debug)]
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
