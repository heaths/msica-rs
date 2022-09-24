// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::ffi;
use super::{Record, View, MSIHANDLE, PMSIHANDLE};
use std::ffi::CString;

/// The database for the current install session.
pub struct Database<'a> {
    h: PMSIHANDLE<'a>,
}

impl<'a> Database<'a> {
    /// Returns a [`View`] object that represents the query specified by a
    /// [SQL string](https://docs.microsoft.com/windows/win32/msi/sql-syntax).
    pub fn open_view(&'a self, sql: &str) -> View<'a> {
        unsafe {
            let h = MSIHANDLE::null();
            let sql = CString::new(sql).unwrap();

            // TODO: Return Result<View<'a>, ?>.
            ffi::MsiDatabaseOpenView(*self.h, sql.as_ptr(), &h);

            View::from(h)
        }
    }

    /// Returns a [`Record`] object containing the table name in field 0 and the column names
    /// (comprising the primary keys) in succeeding fields corresponding to their column numbers.
    ///
    /// The field count of the record is the count of primary key columns.
    pub fn primary_keys(&'a self, table: &str) -> Record<'a> {
        unsafe {
            let h = MSIHANDLE::null();
            let table = CString::new(table).unwrap();

            // TODO: Return Result<View<'a>, ?>.
            ffi::MsiDatabaseGetPrimaryKeys(*self.h, table.as_ptr(), &h);

            Record::from(h)
        }
    }
}

impl<'a> From<MSIHANDLE> for Database<'a> {
    fn from(h: MSIHANDLE) -> Self {
        Database { h: h.to_owned() }
    }
}
