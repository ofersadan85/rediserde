use std::{str::Utf8Error, string::FromUtf8Error};

#[derive(Debug, Clone)]
pub enum Error {
    SerializeError(String),
    DeserializeError(String),
    UnexpectedEnd,
    UnexpectedByte { expected: String, found: char },
    UnrecognizedStart,
    InvalidUtf8,
    ExpectedLength,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializeError(msg) => {
                write!(f, "Failed to serialize RESP data: {msg}")
            }
            Self::DeserializeError(msg) => {
                write!(f, "Failed to deserialize RESP data: {msg}")
            }
            Error::UnexpectedEnd => write!(f, "Unexpected end of input"),
            Error::UnexpectedByte { expected, found } => {
                write!(f, "Unexpected byte: expected {expected}, found {found}",)
            }
            Error::UnrecognizedStart => write!(f, "Unrecognized start of RESP data"),
            Error::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence in RESP data"),
            Error::ExpectedLength => write!(f, "Expected a length for following items"),
        }
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::SerializeError(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::DeserializeError(msg.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_err: FromUtf8Error) -> Self {
        Self::InvalidUtf8
    }
}

impl From<Utf8Error> for Error {
    fn from(_err: Utf8Error) -> Self {
        Self::InvalidUtf8
    }
}
