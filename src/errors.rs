// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use thiserror::Error;

use crate::Record;

/// Errors returned by this crate.
#[derive(Error, Debug)]
pub enum Error {
    /// A Windows error code returned from installer functions.
    #[error("error code {0}")]
    ErrorCode(u32),

    /// An error [`Record`] containing more information.
    #[error("installer error: {0}")]
    ErrorRecord(Record),

    /// A possible error value when converting a `String` from a UTF-8 byte vector.
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

/// Results returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "nightly")]
pub mod experimental {
    use crate::ffi;
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
    /// use msica::{Session, experimental::CustomActionResult};
    ///
    /// #[no_mangle]
    /// pub extern "C" fn MyCustomAction(h: MSIHANDLE) -> CustomActionResult {
    ///     let session = Session::from(h);
    ///     let s = OsString::from("hopefully some properly UTF8-encoded string")?;
    ///
    ///     // Do something with `s`.
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
}
