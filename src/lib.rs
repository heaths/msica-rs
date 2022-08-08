// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#![allow(dead_code)]
#![doc = include_str!("../README.md")]

// Fail fast on non-Windows platforms.
#[cfg(not(target_os = "windows"))]
compile_error!("supported on windows only");

// See https://docs.microsoft.com/windows/win32/msi/automation-interface-reference

mod database;
mod ffi;
mod record;
mod session;

pub use database::Database;
pub use ffi::{
    ERROR_FUNCTION_NOT_CALLED, ERROR_INSTALL_FAILURE, ERROR_INSTALL_USEREXIT, ERROR_NO_MORE_ITEMS,
    ERROR_SUCCESS,
};
pub use record::{Field, Record};
pub use session::Session;

use std::{fmt::Debug, ops::Deref};

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

/// Run modes passed to `Session::mode`.
#[repr(u32)]
pub enum RunMode {
    /// Administrative mode install, else product install.
    Admin = 0,
    /// Advertise mode of install.
    Advertise = 1,
    ///Maintenance mode database loaded.
    Maintenance = 2,
    /// Rollback is enabled.
    RollbackEnabled = 3,
    /// Log file is active.
    LogEnabled = 4,
    /// Executing or spooling operations.
    Operations = 5,
    /// Reboot is needed.
    RebootAtEnd = 6,
    /// Reboot is needed to continue installation
    RebootNow = 7,
    /// Installing files from cabinets and files using Media table.
    Cabinet = 8,
    /// Source files use only short file names.
    SourceShortNames = 9,
    /// Target files are to use only short file names.
    TargetShortNames = 10,
    /// Operating system is Windows 98/95.
    Windows9x = 12,
    /// Operating system supports advertising of products.
    ZawEnabled = 13,
    /// Deferred custom action called from install script execution.
    Scheduled = 16,
    /// Deferred custom action called from rollback execution script.
    Rollback = 17,
    /// Deferred custom action called from commit execution script.
    Commit = 18,
}

/// Gets the last Windows Installer error for the current process.
///
/// # Example
///
/// ```
/// use msica::*;
///
/// if let Some(error) = last_error_record() {
///     println!("last error: {}", error.format_text());
/// }
/// ```
pub fn last_error_record() -> Option<Record> {
    unsafe {
        match ffi::MsiGetLastErrorRecord() {
            h if !h.is_null() => Some(h.into()),
            _ => None,
        }
    }
}

/// A Windows Installer handle. This handle is not automatically closed.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MSIHANDLE(u32);

impl MSIHANDLE {
    fn null() -> MSIHANDLE {
        MSIHANDLE(0)
    }

    fn to_owned(&self) -> PMSIHANDLE {
        PMSIHANDLE(*self)
    }

    fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl From<u32> for MSIHANDLE {
    fn from(h: u32) -> Self {
        MSIHANDLE(h)
    }
}

impl Debug for MSIHANDLE {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MSIHANDLE ({})", self.0)
    }
}

impl Deref for MSIHANDLE {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A Windows Installer handle. This handle is automatically closed when dropped.
#[repr(transparent)]
struct PMSIHANDLE(MSIHANDLE);

impl Debug for PMSIHANDLE {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MSIHANDLE ({})", *self.0)
    }
}

impl Drop for PMSIHANDLE {
    fn drop(&mut self) {
        unsafe {
            ffi::MsiCloseHandle(**self);
        }
    }
}

impl Deref for PMSIHANDLE {
    type Target = MSIHANDLE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
