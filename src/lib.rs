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
mod view;

pub use database::Database;
pub use ffi::{
    ERROR_FUNCTION_NOT_CALLED, ERROR_INSTALL_FAILURE, ERROR_INSTALL_USEREXIT, ERROR_NO_MORE_ITEMS,
    ERROR_SUCCESS,
};
pub use record::{Field, Record};
pub use session::Session;
pub use view::View;

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

    /// Updates an existing record. Nonprimary keys only. Must first call `fetch` on [`View`].
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
pub fn last_error_record<'a>() -> Option<Record> {
    unsafe {
        match ffi::MsiGetLastErrorRecord() {
            h if !h.is_null() => Some(Record::from_handle(h)),
            _ => None,
        }
    }
}
