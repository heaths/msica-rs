// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::ffi;
use super::{ModifyMode, Record, MSIHANDLE, PMSIHANDLE};
use std::ops::Deref;

/// The `View` object represents a result set obtained when processing a query
/// using the `OpenView` method of the [`Database`] object. Before any data can be transferred,
/// the query must be executed using the `execute` method, passing to it all replaceable parameters
/// designated within the SQL query string.
///
/// The query may be executed again, with different parameters if needed,
/// but only after freeing the result set either by fetching all the records or by calling the `close` method.
pub struct View<'a> {
    h: PMSIHANDLE<'a>,
}

impl<'a> View<'a> {
    /// Releases the result set for an executed view.
    pub fn close(&self) {
        unsafe {
            ffi::MsiViewClose(*self.h);
        }
    }

    /// The `execute` method uses the question mark token to represent parameters in an SQL statement.
    /// For more information, see [SQL syntax](https://docs.microsoft.com/windows/win32/msi/sql-syntax).
    ///
    /// The values of these parameters are passed in as the corresponding fields of a parameter record.
    pub fn execute(&self, record: Option<Record<'a>>) {
        unsafe {
            let h = match record {
                Some(r) => *r,
                None => MSIHANDLE::null(),
            };

            ffi::MsiViewExecute(*self.h, h);
        }
    }

    /// Updates a fetched record.
    ///
    /// You can pass `Update` or `Delete` with a record immediately after using `Insert`, `InsertTemporary`, or `Seek` provided you have *not* modified the 0th field of the inserted or sought record.
    ///
    /// You cannot fetch a record that contains binary data from one database and then use that record to insert the data into another database.
    ///
    /// Note that custom actions can only add, modify, or remove temporary rows, columns, or tables from a database.
    /// Custom actions cannot modify persistent data in a database, such as data that is a part of the database stored on disk.
    pub fn modify(&self, mode: ModifyMode, record: &Record) {
        unsafe {
            ffi::MsiViewModify(*self.h, mode, **record);
        }
    }
}

impl<'a> Drop for View<'a> {
    fn drop(&mut self) {
        self.close();
    }
}

impl<'a> Deref for View<'a> {
    type Target = MSIHANDLE;

    fn deref(&self) -> &Self::Target {
        &*self.h
    }
}

impl<'a> From<MSIHANDLE> for View<'a> {
    fn from(h: MSIHANDLE) -> Self {
        View { h: h.to_owned() }
    }
}

pub struct ViewIterator<'a> {
    view: View<'a>,
    record: Option<Record<'a>>,
}

impl<'a> Iterator for ViewIterator<'a> {
    type Item = Record<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            // Reuse Record if already allocated.
            let mut h = match self.record {
                Some(ref r) => **r,
                None => MSIHANDLE::null(),
            };

            ffi::MsiViewFetch(*self.view, &mut h);

            if h.is_null() {
                return None;
            }

            self.record = Some(Record::from(h));
            self.record.clone()
        }
    }
}

impl<'a> IntoIterator for View<'a> {
    type Item = Record<'a>;
    type IntoIter = ViewIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ViewIterator {
            view: self,
            record: None,
        }
    }
}
