// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#![allow(dead_code)]
#![cfg_attr(feature = "nightly", feature(try_trait_v2))]
#![doc = include_str!("../README.md")]

// Fail fast on non-Windows platforms.
#[cfg(not(target_os = "windows"))]
compile_error!("supported on windows only");

// See https://docs.microsoft.com/windows/win32/msi/automation-interface-reference
// for inspiration for the shape of this API.

mod database;
mod errors;
mod ffi;
mod record;
mod session;
mod view;

pub use database::Database;
#[cfg(feature = "nightly")]
pub use errors::experimental::CustomActionResult;
pub use errors::{Error, ErrorKind, Result};
pub use record::{Field, Record};
pub use session::{MessageType, RunMode, Session};
pub use view::{ModifyMode, View};

/// Gets the last Windows Installer error for the current process.
///
/// # Example
///
/// ```
/// if let Some(error) = msica::last_error_record() {
///     println!("last error: {}", error.format_text()?);
/// }
/// # Ok::<(), msica::Error>(())
/// ```
pub fn last_error_record<'a>() -> Option<Record> {
    unsafe {
        match ffi::MsiGetLastErrorRecord() {
            h if !h.is_null() => Some(Record::from_handle(h)),
            _ => None,
        }
    }
}
