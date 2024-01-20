// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#![cfg(feature = "nightly")]
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

    /// Skip remaining actions. Not an error.
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

#[allow(clippy::from_over_into)]
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
    fn from_residual(residual: CustomActionResultCode) -> Self {
        match residual.0.into() {
            ffi::ERROR_NO_MORE_ITEMS => CustomActionResult::Skip,
            ffi::ERROR_INSTALL_USEREXIT => CustomActionResult::Cancel,
            ffi::ERROR_INSTALL_FAILURE => CustomActionResult::Fail,
            ffi::ERROR_FUNCTION_NOT_CALLED => CustomActionResult::NotExecuted,
            code => panic!("unexpected error code {}", code),
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

impl<E: std::error::Error> FromResidual<std::result::Result<Infallible, E>> for CustomActionResult {
    default fn from_residual(_: std::result::Result<Infallible, E>) -> Self {
        CustomActionResult::Fail
    }
}

#[cfg(test)]
mod tests {
    use crate::Record;

    use super::*;

    #[test]
    fn from_u32() {
        assert_eq!(CustomActionResult::Succeed, CustomActionResult::from(0u32));
        assert_eq!(CustomActionResult::Skip, CustomActionResult::from(259u32));
        assert_eq!(
            CustomActionResult::Cancel,
            CustomActionResult::from(1602u32)
        );
        assert_eq!(
            CustomActionResult::NotExecuted,
            CustomActionResult::from(1626u32)
        );
        assert_eq!(CustomActionResult::Fail, CustomActionResult::from(1603u32));
        assert_eq!(CustomActionResult::Fail, CustomActionResult::from(1u32));
    }

    #[test]
    fn into_u32() {
        assert_eq!(0u32, Into::<u32>::into(CustomActionResult::Succeed));
        assert_eq!(259u32, Into::<u32>::into(CustomActionResult::Skip));
        assert_eq!(1602u32, Into::<u32>::into(CustomActionResult::Cancel));
        assert_eq!(1603u32, Into::<u32>::into(CustomActionResult::Fail));
        assert_eq!(1626u32, Into::<u32>::into(CustomActionResult::NotExecuted));
    }

    #[test]
    fn from_residual_custom_action_result() {
        let f = || -> CustomActionResult { CustomActionResult::Skip };
        assert_eq!(259u32, f().into());
    }

    #[test]
    fn from_residual_error() {
        let f = || -> CustomActionResult { Err(Error::from_error_code(1602u32))? };
        assert_eq!(1602u32, f().into());

        let r = Record::with_fields(Some("error"), vec![]).expect("failed to create record");
        let f = || -> CustomActionResult { Err(Error::from_error_record(r))? };
        assert_eq!(1603u32, f().into());
    }

    #[test]
    fn from_residual_std_error() {
        let f = || -> CustomActionResult { Err(std::io::Error::from_raw_os_error(5))? };
        assert_eq!(1603u32, f().into());
    }
}
