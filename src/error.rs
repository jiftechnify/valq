/// An error type returned from `Result`-returning macros provided by `valq` crate.
#[derive(Debug)]
pub enum Error {
    /// No value found at the specified path.
    ValueNotFoundAtPath(String),
    /// Casting a value with `->` operator (translates to `as_***()`/`as_***_mut()`) failed.
    AsCastFailed(String),
    /// Deserialization with `>>` operator failed.
    DeserializationFailed(Box<dyn std::error::Error>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error;
        match self {
            Error::ValueNotFoundAtPath(path) => {
                write!(f, "value not found at the path: {}", path)
            }
            Error::AsCastFailed(conv_name) => {
                write!(f, "casting with {}() failed", conv_name)
            }
            Error::DeserializationFailed(err) => {
                write!(f, "failed to deserialize the queried value: {}", err)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error;
        match self {
            Error::DeserializationFailed(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}
