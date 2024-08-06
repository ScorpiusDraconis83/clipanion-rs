#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    AmbiguousSyntax(Vec<usize>),
    Custom(usize, String),
    UnknownSyntax(usize, String),
    InternalError,
    NotFound(Vec<usize>),
}

#[derive(Debug)]
pub enum BuildError {
    MultipleRestParameters,
    OptionalParametersAfterRest,
    OptionalParametersAfterTrailingPositionals,
    RestAfterTrailingPositionals,
    ArityTooHighForNonBindingOption,
}
