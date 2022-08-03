// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#![allow(dead_code)]

// See https://docs.microsoft.com/windows/win32/msi/automation-interface-reference

mod ffi;
mod record;
mod session;

pub use ffi::{
    ERROR_FUNCTION_NOT_CALLED, ERROR_INSTALL_FAILURE, ERROR_INSTALL_USEREXIT, ERROR_NO_MORE_ITEMS,
    ERROR_SUCCESS,
};
pub use record::{Field, Record};
pub use session::Session;

use std::fmt::Debug;

/// Message types that can be processed by a custom action.
#[repr(C)]
pub enum MessageType {
    Error = 0x0100_0000,
    Warning = 0x0200_0000,
    User = 0x0300_0000,
    Info = 0x0400_0000,
    Progress = 0x0a00_0000,
    CommonData = 0x0b00_0000,
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
        PMSIHANDLE(self.0)
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

/// A Windows Installer handle. This handle is automatically closed when dropped.
#[repr(transparent)]
struct PMSIHANDLE(u32);

impl Debug for PMSIHANDLE {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MSIHANDLE ({})", self.0)
    }
}

impl Drop for PMSIHANDLE {
    fn drop(&mut self) {
        unsafe {
            ffi::MsiCloseHandle(self.into());
        }
    }
}

impl Into<MSIHANDLE> for PMSIHANDLE {
    fn into(self) -> MSIHANDLE {
        MSIHANDLE(self.0)
    }
}
impl Into<MSIHANDLE> for &mut PMSIHANDLE {
    fn into(self) -> MSIHANDLE {
        MSIHANDLE(self.0)
    }
}
