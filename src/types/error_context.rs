use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorContext {
    Message(String),
    Location { file: String, line: u32 },
    Tag(String),
    Metadata { key: String, value: String },
}

impl ErrorContext {
    #[inline]
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self::Message(message.into())
    }

    #[inline]
    pub fn location(file: &str, line: u32) -> Self {
        Self::Location {
            file: file.to_string(),
            line,
        }
    }

    #[inline]
    pub fn tag<S: Into<String>>(tag: S) -> Self {
        Self::Tag(tag.into())
    }

    #[inline]
    pub fn metadata<K: Into<String>, V: Into<String>>(key: K, value: V) -> Self {
        Self::Metadata {
            key: key.into(),
            value: value.into(),
        }
    }

    #[inline]
    pub fn message(&self) -> String {
        match self {
            Self::Message(s) => s.clone(),
            Self::Location { file, line } => format!("at {}:{}", file, line),
            Self::Tag(t) => format!("[{}]", t),
            Self::Metadata { key, value } => format!("{}={}", key, value),
        }
    }
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ErrorContext {}
