// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#![allow(dead_code)]
#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "nightly", feature(try_trait_v2))]

// Fail fast on non-Windows platforms.
#[cfg(not(target_os = "windows"))]
compile_error!("supported on windows only");

// See https://docs.microsoft.com/windows/win32/msi/automation-interface-reference

mod database;
mod errors;
mod ffi;
mod record;
mod session;
mod view;

pub use database::Database;
#[cfg(feature = "nightly")]
pub use errors::experimental::CustomActionResult;
pub use errors::{Error, Result};
pub use record::{Field, Record};
pub use session::{RunMode, Session};
pub use view::{ModifyMode, View};

/// Message types that can be processed by a custom action.
#[repr(u32)]
pub enum MessageType {
    Error = 0x0100_0000,
    Warning = 0x0200_0000,
    User = 0x0300_0000,
    Info = 0x0400_0000,
    Progress = 0x0a00_0000,
    CommonData = 0x0b00_0000,
}

/// Gets the last Windows Installer error for the current process.
///
/// # Example
///
/// ```
/// use msica::*;
///
/// if let Some(error) = last_error_record() {
///     println!("last error: {}", error.format_text().unwrap());
/// }
/// ```
pub fn last_error_record<'a>() -> Option<Record> {
    unsafe {
        match ffi::MsiGetLastErrorRecord() {
            h if !h.is_null() => Some(Record::from_handle(h)),
            _ => None,
        }
    }
}
