// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::{MessageType, RunMode, MSIHANDLE};
use std::{ops::Not, os::raw::c_char};

pub(crate) type LPSTR = *mut c_char;
pub(crate) type LPCSTR = *const c_char;

pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_NO_MORE_ITEMS: u32 = 259;
pub const ERROR_INSTALL_USEREXIT: u32 = 1602;
pub const ERROR_INSTALL_FAILURE: u32 = 1603;
pub const ERROR_FUNCTION_NOT_CALLED: u32 = 1626;

pub(crate) const ERROR_MORE_DATA: u32 = 234;

// cspell:ignore pcch
#[link(name = "msi")]
extern "C" {
    pub fn MsiCloseHandle(hAny: MSIHANDLE) -> u32;

    pub fn MsiCreateRecord(cParams: u32) -> MSIHANDLE;

    #[link_name = "MsiDoActionA"]
    pub fn MsiDoAction(hInstall: MSIHANDLE, szAction: LPCSTR) -> u32;

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
}

#[derive(Copy, Clone)]
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

impl From<bool> for BOOL {
    fn from(value: bool) -> Self {
        if value {
            BOOL(1)
        } else {
            BOOL(0)
        }
    }
}

impl Not for BOOL {
    type Output = Self;
    fn not(self) -> Self::Output {
        if self.as_bool() {
            BOOL(0)
        } else {
            BOOL(1)
        }
    }
}

impl PartialEq<bool> for BOOL {
    fn eq(&self, other: &bool) -> bool {
        self.as_bool() == *other
    }
}
