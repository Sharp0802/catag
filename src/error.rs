use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use crate::iter::At;

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("expected character")]
    NoChar,
    #[error("expected escape prefix")]
    NoEscapePrefix,
    #[error("invalid escape prefix")]
    InvalidEscapePrefix,
    #[error("no hex digit")]
    NoHexDigit,
    #[error("insufficient hex digit")]
    InsufficientHexDigit,
    #[error("no left-side pattern")]
    NoLeftPattern,
    #[error("no left-side character")]
    NoLeftChar,
    #[error("no right-side character")]
    NoRightChar,
    #[error("mismatched paren")]
    MismatchedParen,
    #[error("unexpected semicolon")]
    UnexpectedSemicolon,
    #[error("expected delimiter")]
    NoDelimiter,
    #[error("limiter not closed")]
    LimiterNotClosed,
    #[error("invalid limiter")]
    InvalidLimiter,
    #[error("unexpected '{0}'")]
    UnexpectedChar(char),
    #[error("unexpected EOF")]
    UnexpectedEof,
    #[error("expected identifier")]
    NoIdent,
    #[error("expected equal sign")]
    NoEq,
    #[error("name confliction")]
    ConflictName,
    #[error("invalid unicode")]
    InvalidUnicode,
}

impl ErrorKind {
    pub fn at(self, at: At) -> Error {
        Error {
            at,
            kind: self
        }
    }
}

pub struct Error {
    at: At,
    kind: ErrorKind,
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.at, self.kind)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.at, self.kind)
    }
}
