// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::ffi;
use super::{Database, MessageType, Record, RunMode, MSIHANDLE};
use std::ffi::CString;
use std::marker::PhantomData;
use std::ops::Deref;

/// A Windows Installer session passed as an [`MSIHANDLE`] to custom actions.
///
/// # Example
///
/// ```no_run
/// use msica::*;
/// const ERROR_SUCCESS: u32 = 0;
///
/// #[no_mangle]
/// pub extern "C" fn MyCustomAction(h: MSIHANDLE) -> u32 {
///     let session = Session::from(h);
///     let record = Record::with_fields(
///         Some("this is [1] [2]"),
///         vec![Field::IntegerData(1), Field::StringData("example".to_owned())],
///     );
///     session.message(MessageType::User, &record);
///     ERROR_SUCCESS
/// }
/// ```
pub struct Session<'a> {
    h: MSIHANDLE,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Session<'a> {
    /// Returns the active database for the installation. This function returns a read-only [`Database`].
    pub fn database(&'a self) -> Database<'a> {
        unsafe {
            let h = ffi::MsiGetActiveDatabase(self.h);
            Database::from(h)
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
    /// pub extern "C" fn MyCustomAction(h: MSIHANDLE) -> u32 {
    ///     let session = Session::from(h);
    ///     for i in 0..5 {
    ///         session.do_deferred_action("MyDeferredCustomAction", &i.to_string())
    ///     }
    ///     ERROR_SUCCESS
    /// }
    ///
    /// #[no_mangle]
    /// pub extern "C" fn MyDeferredCustomAction(h: MSIHANDLE) -> u32 {
    ///     let session = Session::from(h);
    ///     let data = session.property("CustomActionData");
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
        unsafe { ffi::MsiProcessMessage(self.h, kind, **record) }
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
    /// pub extern "C" fn MyCustomAction(h: MSIHANDLE) -> u32 {
    ///     let session = Session::from(h);
    ///     if !session.mode(RunMode::Scheduled) {
    ///         session.do_deferred_action("MyCustomAction", "Hello, world!");
    ///     } else {
    ///         let data = session.property("CustomActionData");
    ///         let record = Record::with_fields(Some(data.as_str()), vec![]);
    ///         session.message(MessageType::User, &record);
    ///     }
    ///     ERROR_SUCCESS
    /// }
    /// ```
    pub fn mode(&self, mode: RunMode) -> bool {
        unsafe { ffi::MsiGetMode(self.h, mode).as_bool() }
    }

    /// Gets the value of the named property, or an empty string if undefined.
    pub fn property(&self, name: &str) -> String {
        unsafe {
            // TODO: Return result containing NulError if returned.
            let name = CString::new(name).unwrap();

            let mut value_len = 0u32;
            let value = CString::default();

            if ffi::MsiGetProperty(
                self.h,
                name.as_ptr(),
                value.as_ptr() as ffi::LPSTR,
                &mut value_len as *mut u32,
            ) == ffi::ERROR_MORE_DATA
            {
                let mut value_len = value_len + 1u32;
                let mut value: Vec<u8> = vec![0; value_len as usize];

                ffi::MsiGetProperty(
                    self.h,
                    name.as_ptr(),
                    value.as_mut_ptr() as ffi::LPSTR,
                    &mut value_len as *mut u32,
                );

                value.truncate(value_len as usize);
                return String::from_utf8(value).unwrap();
            }

            String::default()
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

impl<'a> Deref for Session<'a> {
    type Target = MSIHANDLE;

    fn deref(&self) -> &Self::Target {
        &self.h
    }
}

impl<'a> From<MSIHANDLE> for Session<'a> {
    fn from(h: MSIHANDLE) -> Self {
        Session {
            h,
            _phantom: PhantomData,
        }
    }
}
