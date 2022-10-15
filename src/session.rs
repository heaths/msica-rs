// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::ffi;
use crate::{Database, Error, MessageType, Record, Result};
use std::ffi::CString;

/// A Windows Installer session passed as an [`MSIHANDLE`] to custom actions.
///
/// # Example
///
/// ```no_run
/// use msica::*;
///
/// #[no_mangle]
/// pub extern "C" fn MyCustomAction(session: Session) -> CustomActionResult {
///     let record = Record::with_fields(
///         Some("this is [1] [2]"),
///         vec![Field::IntegerData(1), Field::StringData("example".to_owned())],
///     )?;
///     session.message(MessageType::User, &record);
///     CustomActionResult::Succeed
/// }
/// ```
#[repr(transparent)]
pub struct Session {
    h: ffi::MSIHANDLE,
}

impl Session {
    /// Returns the active database for the installation. This function returns a read-only [`Database`].
    pub fn database(&self) -> Database {
        unsafe {
            let h = ffi::MsiGetActiveDatabase(self.h);
            Database::from_handle(h)
        }
    }

    /// Runs the specified immediate custom action, or schedules a deferred custom action.
    /// If `None` the default action is run e.g., `INSTALL`.
    ///
    /// To schedule a deferred custom action with its `CustomActionData`,
    /// call `do_deferred_action`.
    pub fn do_action(&self, action: Option<&str>) {
        unsafe {
            let action = match action {
                Some(s) => CString::new(s).unwrap(),
                None => CString::default(),
            };
            ffi::MsiDoAction(self.h, action.as_ptr());
        }
    }

    /// Sets custom action data and schedules a deferred custom action.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use msica::*;
    /// const ERROR_SUCCESS: u32 = 0;
    ///
    /// #[no_mangle]
    /// pub extern "C" fn MyCustomAction(session: Session) -> u32 {
    ///     for i in 0..5 {
    ///         session.do_deferred_action("MyDeferredCustomAction", &i.to_string())
    ///     }
    ///     ERROR_SUCCESS
    /// }
    ///
    /// #[no_mangle]
    /// pub extern "C" fn MyDeferredCustomAction(session: Session) -> u32 {
    ///     let data = session.property("CustomActionData").expect("failed to get CustomActionData");
    ///     let record = Record::from(data);
    ///     session.message(MessageType::Info, &record);
    ///     ERROR_SUCCESS
    /// }
    /// ```
    pub fn do_deferred_action(&self, action: &str, custom_action_data: &str) {
        self.set_property(action, Some(custom_action_data));
        self.do_action(Some(action));
    }

    /// The numeric language ID used by the current install session.
    pub fn language(&self) -> u16 {
        unsafe { ffi::MsiGetLanguage(self.h) }
    }

    /// Processes a [`Record`] within the [`Session`].
    pub fn message(&self, kind: MessageType, record: &Record) -> i32 {
        unsafe { ffi::MsiProcessMessage(self.h, kind, *record.h) }
    }

    /// Returns a boolean indicating whether the specific property passed into the function is currently set (true) or not set (false).
    ///
    /// # Example
    ///
    /// You could use the same custom action entry point for scheduling and executing deferred actions:
    ///
    /// ```no_run
    /// use msica::*;
    ///
    /// #[no_mangle]
    /// pub extern "C" fn MyCustomAction(session: Session) -> CustomActionResult {
    ///     if !session.mode(RunMode::Scheduled) {
    ///         session.do_deferred_action("MyCustomAction", "Hello, world!");
    ///     } else {
    ///         let data = session.property("CustomActionData")?;
    ///         let record = Record::with_fields(Some(data.as_str()), vec![])?;
    ///         session.message(MessageType::User, &record);
    ///     }
    ///     CustomActionResult::Succeed
    /// }
    /// ```
    pub fn mode(&self, mode: RunMode) -> bool {
        unsafe { ffi::MsiGetMode(self.h, mode).as_bool() }
    }

    /// Gets the value of the named property, or an empty string if undefined.
    pub fn property(&self, name: &str) -> Result<String> {
        unsafe {
            // TODO: Return result containing NulError if returned.
            let name = CString::new(name)?;

            let mut value_len = 0u32;
            let value = CString::default();

            let mut ret = ffi::MsiGetProperty(
                self.h,
                name.as_ptr(),
                value.as_ptr() as ffi::LPSTR,
                &mut value_len as *mut u32,
            );
            if ret != ffi::ERROR_MORE_DATA {
                return Err(Error::from_error_code(ret));
            }

            let mut value_len = value_len + 1u32;
            let mut value: Vec<u8> = vec![0; value_len as usize];

            ret = ffi::MsiGetProperty(
                self.h,
                name.as_ptr(),
                value.as_mut_ptr() as ffi::LPSTR,
                &mut value_len as *mut u32,
            );
            if ret != ffi::ERROR_SUCCESS {
                return Err(Error::from_error_code(ret));
            }

            value.truncate(value_len as usize);
            let text = String::from_utf8(value)?;

            Ok(text)
        }
    }

    /// Sets the value of the named property. Pass `None` to clear the field.
    pub fn set_property(&self, name: &str, value: Option<&str>) {
        unsafe {
            let name = CString::new(name).unwrap();
            let value = match value {
                Some(s) => CString::new(s).unwrap(),
                None => CString::default(),
            };

            ffi::MsiSetProperty(
                self.h,
                name.as_ptr() as ffi::LPCSTR,
                value.as_ptr() as ffi::LPCSTR,
            );
        }
    }
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
