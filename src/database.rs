// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::ffi;
use crate::{Record, View};
use std::ffi::CString;

/// The database for the current install session.
pub struct Database {
    h: ffi::PMSIHANDLE,
}

impl Database {
    /// Returns a [`View`] object that represents the query specified by a
    /// [SQL string](https://docs.microsoft.com/windows/win32/msi/sql-syntax).
    pub fn open_view(&self, sql: &str) -> View {
        unsafe {
            let h = ffi::MSIHANDLE::null();
            let sql = CString::new(sql).unwrap();
            ffi::MsiDatabaseOpenView(*self.h, sql.as_ptr(), &h);

            View::from_handle(h)
        }
    }

    /// Returns a [`Record`] object containing the table name in field 0 and the column names
    /// (comprising the primary keys) in succeeding fields corresponding to their column numbers.
    ///
    /// The field count of the record is the count of primary key columns.
    pub fn primary_keys(&self, table: &str) -> Record {
        unsafe {
            let h = ffi::MSIHANDLE::null();
            let table = CString::new(table).unwrap();
            ffi::MsiDatabaseGetPrimaryKeys(*self.h, table.as_ptr(), &h);

            Record::from_handle(h)
        }
    }

    pub(crate) fn from_handle(h: ffi::MSIHANDLE) -> Self {
        Database { h: h.to_owned() }
    }
}
