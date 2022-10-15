// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::Record;
use std::fmt::Display;
use std::num::NonZeroU32;

/// Results returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
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
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
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

    pub fn from_error_code(code: u32) -> Self {
        Self {
            context: Context::Simple(ErrorKind::ErrorCode(
                NonZeroU32::new(code).expect("expected non-zero error code"),
            )),
        }
    }

    pub fn from_last_error_record() -> Option<Self> {
        crate::last_error_record().map(|record| Self {
            context: Context::Record(record),
        })
    }

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

#[cfg(feature = "nightly")]
pub mod experimental {
    use super::{Error, ErrorKind};
    use crate::ffi;
    use std::convert::Infallible;
    use std::fmt::Display;
    use std::num::NonZeroU32;
    use std::ops::{ControlFlow, FromResidual, Try};

    /// A result to return from a custom action.
    ///
    /// This allows you to use the `?` operator to map any `Result<T, E>` to [`CustomActionResult::Fail`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::ffi::OsString;
    /// use msica::{Session, CustomActionResult};
    ///
    /// #[no_mangle]
    /// pub extern "C" fn MyCustomAction(session: Session) -> CustomActionResult {
    ///     let productName = session.property("ProductName")?;
    ///
    ///     // Do something with `productName`.
    ///
    ///     CustomActionResult::Succeed
    /// }
    /// ```
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[repr(u32)]
    pub enum CustomActionResult {
        /// Completed actions successfully.
        Succeed = ffi::ERROR_SUCCESS,

        /// Skip remaining actions.Not an error.
        Skip = ffi::ERROR_NO_MORE_ITEMS,

        /// User terminated prematurely.
        Cancel = ffi::ERROR_INSTALL_USEREXIT,

        /// Unrecoverable error occurred.
        Fail = ffi::ERROR_INSTALL_FAILURE,

        /// Action not executed.
        NotExecuted = ffi::ERROR_FUNCTION_NOT_CALLED,
    }

    impl Display for CustomActionResult {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let error = match &self {
                Self::Succeed => "succeeded",
                Self::Skip => "skip remaining actions",
                Self::Cancel => "user canceled",
                Self::Fail => "failed",
                Self::NotExecuted => "not executed",
            };

            write!(f, "{}", error)
        }
    }

    impl From<u32> for CustomActionResult {
        fn from(code: u32) -> Self {
            match code {
                ffi::ERROR_SUCCESS => CustomActionResult::Succeed,
                ffi::ERROR_NO_MORE_ITEMS => CustomActionResult::Skip,
                ffi::ERROR_INSTALL_USEREXIT => CustomActionResult::Cancel,
                ffi::ERROR_FUNCTION_NOT_CALLED => CustomActionResult::NotExecuted,
                _ => CustomActionResult::Fail,
            }
        }
    }

    impl Into<u32> for CustomActionResult {
        fn into(self) -> u32 {
            self as u32
        }
    }

    /// This type is an implementation detail and not intended for direct use.
    #[doc(hidden)]
    pub struct CustomActionResultCode(NonZeroU32);

    impl From<CustomActionResult> for CustomActionResultCode {
        fn from(value: CustomActionResult) -> Self {
            CustomActionResultCode(NonZeroU32::new(value.into()).unwrap())
        }
    }

    impl Try for CustomActionResult {
        type Output = u32;
        type Residual = CustomActionResultCode;

        fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
            match self {
                Self::Succeed => ControlFlow::Continue(ffi::ERROR_SUCCESS),
                _ => ControlFlow::Break(self.into()),
            }
        }

        fn from_output(_: Self::Output) -> Self {
            CustomActionResult::Succeed
        }
    }

    impl FromResidual for CustomActionResult {
        fn from_residual(residual: <Self as Try>::Residual) -> Self {
            match residual.0.into() {
                ffi::ERROR_NO_MORE_ITEMS => CustomActionResult::Skip,
                ffi::ERROR_INSTALL_USEREXIT => CustomActionResult::Cancel,
                ffi::ERROR_INSTALL_FAILURE => CustomActionResult::Fail,
                ffi::ERROR_FUNCTION_NOT_CALLED => CustomActionResult::NotExecuted,
                code => panic!("unexpected result code {}", code),
            }
        }
    }

    impl FromResidual<Result<Infallible, Error>> for CustomActionResult {
        fn from_residual(residual: Result<Infallible, Error>) -> Self {
            let error = residual.unwrap_err();
            match error.kind() {
                ErrorKind::ErrorCode(code) => CustomActionResult::from(code.get()),
                _ => CustomActionResult::Fail,
            }
        }
    }
}
