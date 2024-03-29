// Copyright 2024 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#![allow(dead_code)]
#![cfg_attr(feature = "nightly", feature(min_specialization, try_trait_v2))]
#![doc = include_str!("../README.md")]

// Fail fast on non-Windows platforms.
#[cfg(not(target_os = "windows"))]
compile_error!("supported on windows only");

// See https://docs.microsoft.com/windows/win32/msi/automation-interface-reference
// for inspiration for the shape of this API.

mod database;
mod error;
mod ffi;
mod record;
mod session;
mod view;

pub use database::Database;
#[cfg(feature = "nightly")]
pub use error::experimental::CustomActionResult;
pub use error::{Error, ErrorKind, Result};
pub use record::{Field, Record};
pub use session::{MessageType, RunMode, Session};
pub use view::{ModifyMode, View};

pub mod prelude {
    #[cfg(feature = "nightly")]
    pub use crate::error::experimental::CustomActionResult::{self, *};
    // Export objects and enums used in inputs to those objects' methods.
    pub use crate::{
        Database, Error, Field, MessageType, ModifyMode, Record, Result, RunMode, Session, View,
    };
}

/// Gets the last Windows Installer error for the current process.
///
/// # Example
///
/// ```
/// if let Some(record) = msica::last_error_record() {
///     println!("last error: {}", record.format_text()?);
///     return Err(record.into());
/// }
/// # Ok::<(), msica::Error>(())
/// ```
pub fn last_error_record() -> Option<Record> {
    unsafe {
        match ffi::MsiGetLastErrorRecord() {
            h if !h.is_null() => Some(Record::from_handle(h)),
            _ => None,
        }
    }
}
