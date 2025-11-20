use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ErrorContext {
    message: String,
}

impl ErrorContext {
    #[inline]
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ErrorContext {}
