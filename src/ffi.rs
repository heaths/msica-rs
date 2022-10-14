// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::ModifyMode;

use super::{MessageType, RunMode};
use std::{
    fmt::Display,
    ops::{Deref, Not},
    os::raw::c_char,
};

pub(crate) type LPSTR = *mut c_char;
pub(crate) type LPCSTR = *const c_char;

pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_NO_MORE_ITEMS: u32 = 259;
pub const ERROR_INSTALL_USEREXIT: u32 = 1602;
pub const ERROR_INSTALL_FAILURE: u32 = 1603;
pub const ERROR_FUNCTION_NOT_CALLED: u32 = 1626;

pub(crate) const ERROR_MORE_DATA: u32 = 234;
pub(crate) const MSI_NULL_INTEGER: i32 = -0x8000_0000;

// cspell:ignore pcch
#[link(name = "msi")]
extern "C" {
    pub fn MsiCloseHandle(hAny: MSIHANDLE) -> u32;

    pub fn MsiCreateRecord(cParams: u32) -> MSIHANDLE;

    #[link_name = "MsiDatabaseGetPrimaryKeysA"]
    pub fn MsiDatabaseGetPrimaryKeys(
        hDatabase: MSIHANDLE,
        szTableName: LPCSTR,
        hRecord: &MSIHANDLE,
    ) -> u32;

    #[link_name = "MsiDatabaseOpenViewA"]
    pub fn MsiDatabaseOpenView(hDatabase: MSIHANDLE, szQuery: LPCSTR, phView: &MSIHANDLE) -> u32;

    #[link_name = "MsiDoActionA"]
    pub fn MsiDoAction(hInstall: MSIHANDLE, szAction: LPCSTR) -> u32;

    pub fn MsiGetActiveDatabase(hInstall: MSIHANDLE) -> MSIHANDLE;

    pub fn MsiGetLanguage(hInstall: MSIHANDLE) -> u16;

    pub fn MsiGetLastErrorRecord() -> MSIHANDLE;

    pub fn MsiGetMode(hInstall: MSIHANDLE, eRunMode: RunMode) -> BOOL;

    #[link_name = "MsiGetPropertyA"]
    pub fn MsiGetProperty(
        hInstall: MSIHANDLE,
        szName: LPCSTR,
        szValueBuf: LPSTR,
        pcchValueBuf: *mut u32,
    ) -> u32;

    #[link_name = "MsiFormatRecordA"]
    pub fn MsiFormatRecord(
        hInstall: MSIHANDLE,
        hRecord: MSIHANDLE,
        szResultBuf: LPSTR,
        pcchResultBuf: *mut u32,
    ) -> u32;

    pub fn MsiProcessMessage(
        hInstall: MSIHANDLE,
        eMessageType: MessageType,
        hRecord: MSIHANDLE,
    ) -> i32;

    pub fn MsiRecordGetFieldCount(hRecord: MSIHANDLE) -> u32;

    pub fn MsiRecordGetInteger(hRecord: MSIHANDLE, iField: u32) -> i32;

    #[link_name = "MsiRecordGetStringA"]
    pub fn MsiRecordGetString(
        hRecord: MSIHANDLE,
        iField: u32,
        szValueBuf: LPSTR,
        pcchValueBuf: *mut u32,
    ) -> u32;

    pub fn MsiRecordIsNull(hRecord: MSIHANDLE, iField: u32) -> BOOL;

    pub fn MsiRecordSetInteger(hRecord: MSIHANDLE, iField: u32, iValue: i32) -> u32;

    #[link_name = "MsiRecordSetStringA"]
    pub fn MsiRecordSetString(hRecord: MSIHANDLE, iField: u32, szValue: LPCSTR) -> u32;

    #[link_name = "MsiSetPropertyA"]
    pub fn MsiSetProperty(hInstall: MSIHANDLE, szName: LPCSTR, szValue: LPCSTR) -> u32;

    pub fn MsiViewClose(hView: MSIHANDLE) -> u32;

    pub fn MsiViewExecute(hView: MSIHANDLE, hRecord: MSIHANDLE) -> u32;

    pub fn MsiViewFetch(hView: MSIHANDLE, phRecord: &MSIHANDLE) -> u32;

    pub fn MsiViewModify(hView: MSIHANDLE, eModifyMode: ModifyMode, hRecord: MSIHANDLE) -> u32;
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct BOOL(i32);

impl BOOL {
    #[inline]
    pub fn as_bool(self) -> bool {
        self.0 != 0
    }
}

impl Default for BOOL {
    fn default() -> Self {
        BOOL(0)
    }
}

impl Display for BOOL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self.0 {
            0 => "false",
            _ => "true",
        };
        write!(f, "{}", s)
    }
}

impl From<bool> for BOOL {
    fn from(value: bool) -> Self {
        match value {
            true => BOOL(1),
            false => BOOL(0),
        }
    }
}

impl Not for BOOL {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self.as_bool() {
            true => BOOL(0),
            false => BOOL(1),
        }
    }
}

impl PartialEq<bool> for BOOL {
    fn eq(&self, other: &bool) -> bool {
        self.as_bool() == *other
    }
}

/// A Windows Installer handle. This handle is not automatically closed.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct MSIHANDLE(u32);

impl MSIHANDLE {
    pub fn null() -> MSIHANDLE {
        MSIHANDLE(0)
    }

    pub fn to_owned(&self) -> PMSIHANDLE {
        PMSIHANDLE { h: *self }
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl From<u32> for MSIHANDLE {
    fn from(h: u32) -> Self {
        MSIHANDLE(h)
    }
}

impl Display for MSIHANDLE {
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
#[derive(Debug, PartialEq)]
pub struct PMSIHANDLE {
    h: MSIHANDLE,
}

impl Display for PMSIHANDLE {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MSIHANDLE ({})", *self.h)
    }
}

impl Drop for PMSIHANDLE {
    fn drop(&mut self) {
        unsafe {
            MsiCloseHandle(**self);
        }
    }
}

impl Deref for PMSIHANDLE {
    type Target = MSIHANDLE;

    fn deref(&self) -> &Self::Target {
        &self.h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_null() {
        assert!(MSIHANDLE::null().is_null());
    }
}
