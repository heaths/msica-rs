// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::ffi;
use crate::{Error, Record, Result};

#[cfg(doc)]
use crate::Database;

/// The `View` object represents a result set obtained when processing a query
/// using the `OpenView` method of the [`Database`] object. Before any data can be transferred,
/// the query must be executed using the `execute` method, passing to it all replaceable parameters
/// designated within the SQL query string.
///
/// The query may be executed again, with different parameters if needed,
/// but only after freeing the result set either by fetching all the records or by calling the `close` method.
pub struct View {
    h: ffi::PMSIHANDLE,
}

impl View {
    /// Releases the result set for an executed view.
    ///
    /// `close` must be called before `execute` can be called again unless all records have been fetched.
    pub fn close(&self) {
        unsafe {
            ffi::MsiViewClose(*self.h);
        }
    }

    /// The `execute` method uses the question mark token to represent parameters in an SQL statement.
    /// For more information, see [SQL syntax](https://docs.microsoft.com/windows/win32/msi/sql-syntax).
    ///
    /// The values of these parameters are passed in as the corresponding fields of a parameter record.
    ///
    /// `close` must be called before `execute` can be called again unless all records have been fetched.
    pub fn execute(&self, record: Option<Record>) -> Result<()> {
        unsafe {
            let h = match record {
                Some(r) => *r.h,
                None => ffi::MSIHANDLE::null(),
            };

            let ret = ffi::MsiViewExecute(*self.h, h);
            if ret != ffi::ERROR_SUCCESS {
                return Err(
                    Error::from_last_error_record().unwrap_or_else(|| Error::from_error_code(ret))
                );
            }

            Ok(())
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
    pub fn modify(&self, mode: ModifyMode, record: &Record) -> Result<()> {
        unsafe {
            let ret = ffi::MsiViewModify(*self.h, mode, *record.h);
            if ret != ffi::ERROR_SUCCESS {
                return Err(
                    Error::from_last_error_record().unwrap_or_else(|| Error::from_error_code(ret))
                );
            }

            Ok(())
        }
    }

    pub(crate) fn from_handle(h: ffi::MSIHANDLE) -> Self {
        View { h: h.to_owned() }
    }
}

impl Drop for View {
    fn drop(&mut self) {
        self.close();
    }
}

impl Iterator for View {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut h = ffi::MSIHANDLE::null();
            ffi::MsiViewFetch(*self.h, &mut h);

            if h.is_null() {
                return None;
            }

            Some(Record::from_handle(h))
        }
    }
}

/// Modify modes passed to `View::modify`.
#[repr(u32)]
pub enum ModifyMode {
    /// Refreshes the information in the supplied record without changing the position in the result set and without affecting subsequent fetch operations.
    /// The record may then be used for subsequent Update, Delete, and Refresh. All primary key columns of the table must be in the query and the record must have at least as many fields as the query.
    /// Seek cannot be used with multi-table queries. This mode cannot be used with a view containing joins.
    Seek = u32::MAX,

    /// Refreshes the information in the record. Must first call `fetch` on [`View`] with the same record.
    /// Fails for a deleted row. Works with read-write and read-only records.
    Refresh = 0,

    /// Inserts a record. Fails if a row with the same primary keys exists. Fails with a read-only database.
    /// This mode cannot be used with a view containing joins.
    Insert = 1,

    /// Updates an existing record. Non-primary keys only. Must first call `fetch` on [`View`].
    /// Fails with a deleted record. Works only with read-write records.
    Update = 2,

    /// Writes current data in the cursor to a table row. Updates record if the primary keys match an existing row and inserts if they do not match.
    /// Fails with a read-only database. This mode cannot be used with a view containing joins.
    Assign = 3,

    /// Updates or deletes and inserts a record into a table. Must first call `fetch` on [`View`] with the same record.
    /// Updates record if the primary keys are unchanged. Deletes old row and inserts new if primary keys have changed. Fails with a read-only database.
    /// This mode cannot be used with a view containing joins.
    Replace = 4,

    /// Inserts or validates a record in a table. Inserts if primary keys do not match any row and validates if there is a match.
    /// Fails if the record does not match the data in the table. Fails if there is a record with a duplicate key that is not identical. Works only with read-write records.
    /// This mode cannot be used with a view containing joins.
    Merge = 5,

    /// Remove a row from the table. You must first call the `fetch` on [`View`] function with the same record.
    /// Fails if the row has been deleted. Works only with read-write records. This mode cannot be used with a view containing joins.
    Delete = 6,

    /// Inserts a temporary record. The information is not persistent. Fails if a row with the same primary key exists.
    /// Works only with read-write records. This mode cannot be used with a view containing joins.
    InsertTemporary = 7,
}
