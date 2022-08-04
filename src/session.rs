// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::ffi;
use super::{MessageType, Record, MSIHANDLE};
use std::ffi::CString;
use std::ops::Deref;

/// A Windows Installer session passed as an [`MSIHANDLE`] to custom actions.
///
/// # Example
///
/// ```no_run
/// use msica::*;
///
/// #[no_mangle]
/// pub extern "C" fn MyCustomAction(h: MSIHANDLE) -> u32 {
///     const ERROR_SUCCESS: u32 = 0;
///
///     let session = Session::from(h);
///     let record = Record::with_fields(
///         Some("this is [1] [2]".to_owned()),
///         vec![Field::IntegerData(1), Field::StringData("example".to_owned())],
///     );
///     session.message(MessageType::User, &record);
///
///     ERROR_SUCCESS
/// }
/// ```
pub struct Session {
    h: MSIHANDLE,
    owned: bool,
}

impl Session {
    /// Processes a [`Record`] within the [`Session`].
    pub fn message(&self, kind: MessageType, record: &Record) -> i32 {
        unsafe { ffi::MsiProcessMessage(self.h, kind as u32, record.into()) }
    }

    /// Gets the value of the named property, or an empty string if undefined.
    pub fn property(&self, name: &str) -> String {
        unsafe {
            // TODO: Return result containing NulError if returned.
            let name = CString::new(name).unwrap();

            let mut value_len = 0u32;
            let value = CString::default();

            if ffi::MsiGetProperty(
                self.h,
                name.as_ptr(),
                value.as_ptr() as ffi::LPSTR,
                &mut value_len as *mut u32,
            ) == ffi::ERROR_MORE_DATA
            {
                let mut value_len = value_len + 1u32;
                let mut value: Vec<u8> = vec![0; value_len as usize];

                ffi::MsiGetProperty(
                    self.h,
                    name.as_ptr(),
                    value.as_mut_ptr() as ffi::LPSTR,
                    &mut value_len as *mut u32,
                );

                value.truncate(value_len as usize);
                return String::from_utf8(value).unwrap();
            }

            String::default()
        }
    }
}

impl Deref for Session {
    type Target = MSIHANDLE;

    fn deref(&self) -> &Self::Target {
        &self.h
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                ffi::MsiCloseHandle(self.h);
            }
        }
    }
}

impl From<MSIHANDLE> for Session {
    fn from(h: MSIHANDLE) -> Self {
        Session { h, owned: false }
    }
}
