// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::Record;
use std::fmt::Display;
use std::num::{NonZeroU32, TryFromIntError};

pub mod experimental;

/// Results returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// A Windows error code.
    ErrorCode(NonZeroU32),

    /// A [`Record`] containing Windows Installer error information.
    ErrorRecord,

    /// An error converting data.
    DataConversion,

    /// Any other type of error.
    Other,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::ErrorCode(err) => write!(f, "ErrorCode({})", err),
            ErrorKind::ErrorRecord => write!(f, "ErrorRecord"),
            ErrorKind::DataConversion => write!(f, "DataConversion"),
            ErrorKind::Other => write!(f, "Other"),
        }
    }
}

/// Errors returned by this crate.
#[derive(Debug)]
pub struct Error {
    context: Context,
}

impl Error {
    pub(crate) fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self {
            context: Context::Custom(Custom {
                kind,
                error: error.into(),
            }),
        }
    }

    pub(crate) fn from_error_code(code: u32) -> Self {
        Self {
            context: Context::Simple(ErrorKind::ErrorCode(
                NonZeroU32::new(code).expect("expected non-zero error code"),
            )),
        }
    }

    pub(crate) fn from_error_record(record: Record) -> Self {
        Self {
            context: Context::Record(record),
        }
    }

    pub(crate) fn from_last_error_record() -> Option<Self> {
        crate::last_error_record().map(Error::from_error_record)
    }

    /// Gets the [`ErrorKind`] of this `Error`.
    pub fn kind(&self) -> &ErrorKind {
        match &self.context {
            Context::Simple(kind) => kind,
            Context::Record(..) => &ErrorKind::ErrorRecord,
            Context::Custom(Custom { kind, .. }) => kind,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.context {
            Context::Simple(kind) => write!(f, "{}", kind),
            Context::Record(record) => write!(f, "{}", record),
            Context::Custom(Custom { error, .. }) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.context {
            Context::Custom(Custom { error, .. }) => error.source(),
            _ => None,
        }
    }
}

impl From<TryFromIntError> for Error {
    fn from(error: TryFromIntError) -> Self {
        Error::new(ErrorKind::DataConversion, error)
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(error: std::ffi::NulError) -> Self {
        Error::new(ErrorKind::DataConversion, error)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Error::new(ErrorKind::DataConversion, error)
    }
}

impl From<Record> for Error {
    fn from(record: Record) -> Self {
        Error::from_error_record(record)
    }
}

#[derive(Debug)]
enum Context {
    Simple(ErrorKind),
    Record(Record),
    Custom(Custom),
}

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn std::error::Error + Send + Sync>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Field;
    use std::ffi::CString;

    #[test]
    fn from_error_code() {
        let error = Error::from_error_code(1603);
        assert_eq!(
            &ErrorKind::ErrorCode(NonZeroU32::new(1603).unwrap()),
            error.kind()
        );
        assert_eq!("ErrorCode(1603)", error.to_string());
    }

    #[test]
    fn from_record() {
        let record = Record::with_fields(
            Some("error [1]"),
            vec![Field::StringData("text".to_owned())],
        )
        .expect("failed to create record");
        let error: Error = record.into();
        assert_eq!(&ErrorKind::ErrorRecord, error.kind());
        assert_eq!("error text", error.to_string());
    }

    #[test]
    // cspell:ignore tryfrominterror
    fn from_tryfrominterror() {
        let error: Error = u8::try_from(u32::MAX).unwrap_err().into();
        assert_eq!(&ErrorKind::DataConversion, error.kind());
        assert_ne!("DataConversion", error.to_string());
    }

    #[test]
    // cspell:ignore nulerror
    fn from_nulerror() {
        let error: Error = CString::new("t\0est").unwrap_err().into();
        assert_eq!(&ErrorKind::DataConversion, error.kind());
        assert_ne!("DataConversion", error.to_string());
    }

    #[test]
    // cspell:ignore fromutf8error
    fn from_fromutf8error() {
        let error: Error = String::from_utf8(vec![0, 159, 146, 150])
            .unwrap_err()
            .into();
        assert_eq!(&ErrorKind::DataConversion, error.kind());
        assert_ne!("DataConversion", error.to_string());
    }
}
