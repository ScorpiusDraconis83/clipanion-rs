#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    AmbiguousSyntax,
    UnknownSyntax(String),
}

#[derive(Debug)]
pub enum BuildError {
    MultipleRestParameters,
    OptionalParametersAfterRest,
    OptionalParametersAfterTrailingPositionals,
    RestAfterTrailingPositionals,
    ArityTooHighForNonBindingOption,
}
