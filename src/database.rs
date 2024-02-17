// Copyright 2024 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::ffi;
use crate::{Error, Record, Result, View};
use std::ffi::CString;

/// The database for the current install session.
pub struct Database {
    h: ffi::PMSIHANDLE,
}

impl Database {
    /// Returns a [`View`] object that represents the query specified by a
    /// [SQL string](https://docs.microsoft.com/windows/win32/msi/sql-syntax).
    pub fn open_view(&self, sql: &str) -> Result<View> {
        unsafe {
            let h = ffi::MSIHANDLE::null();
            let sql = CString::new(sql)?;
            let ret = ffi::MsiDatabaseOpenView(*self.h, sql.as_ptr(), &h);
            if ret != ffi::ERROR_SUCCESS {
                return Err(
                    Error::from_last_error_record().unwrap_or_else(|| Error::from_error_code(ret))
                );
            }

            Ok(View::from_handle(h))
        }
    }

    /// Returns a [`Record`] object containing the table name in field 0 and the column names
    /// (comprising the primary keys) in succeeding fields corresponding to their column numbers.
    ///
    /// The field count of the record is the count of primary key columns.
    pub fn primary_keys(&self, table: &str) -> Result<Record> {
        unsafe {
            let h = ffi::MSIHANDLE::null();
            let table = CString::new(table)?;
            let ret = ffi::MsiDatabaseGetPrimaryKeys(*self.h, table.as_ptr(), &h);
            if ret != ffi::ERROR_SUCCESS {
                return Err(Error::from_error_code(ret));
            }

            Ok(Record::from_handle(h))
        }
    }

    pub(crate) fn from_handle(h: ffi::MSIHANDLE) -> Self {
        Database { h: h.to_owned() }
    }
}
