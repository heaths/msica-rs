// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::ffi;
use super::{MSIHANDLE, PMSIHANDLE};
use std::ffi::CString;

/// A field in a [`Record`].
pub enum Field {
    /// A string field in a [`Record`].
    String(String),

    /// An integer field in a [`Record`].
    Integer(i32),

    /// A null field in a [`Record`].
    Null,
}

impl From<&str> for Field {
    fn from(s: &str) -> Self {
        Field::String(s.to_owned())
    }
}

/// A collection of [`Field`] containing strings, integers, and byte streams.
pub struct Record {
    h: PMSIHANDLE,
}

impl Record {
    /// Creates an empty [`Record`] with capacity for the count of fields specified.
    ///
    /// Field indices are 1-based.
    pub fn new(field_count: u32) -> Self {
        unsafe {
            let h = ffi::MsiCreateRecord(field_count);
            Record { h: h.to_owned() }
        }
    }

    /// Creates a [`Record`] with optional text in field 0, with additional fields
    /// containing strings, integers, and byte streams.
    ///
    /// Field indices are 1-based.
    ///
    /// # Example
    ///
    /// ```
    /// use msica::{Field, Record};
    ///
    /// let record = Record::with_fields(
    ///     Some("this is [1] [2]".to_owned()),
    ///     vec![Field::Integer(1), Field::String("example".to_owned())]);
    /// ```
    pub fn with_fields(text: Option<String>, fields: Vec<Field>) -> Self {
        unsafe {
            let h = ffi::MsiCreateRecord(fields.len() as u32);
            let mut record = Record { h: h.to_owned() };

            if let Some(text) = text {
                record.set_string(0, &text);
            }

            for (idx, field) in fields.iter().enumerate() {
                let idx: u32 = idx.try_into().unwrap();
                match field {
                    Field::String(s) => record.set_string(idx + 1, s),
                    Field::Integer(i) => record.set_integer(idx + 1, *i),
                    Field::Null => {}
                }
            }

            record
        }
    }

    /// Gets the count of fields in the record.
    pub fn field_count(&self) -> u32 {
        unsafe { ffi::MsiRecordGetFieldCount(self.into()) }
    }

    /// Formats the template string in field 0 with the remaining fields.
    ///
    /// Specify 1-based field indices using square braces.
    ///
    /// You can also use curly braces such that any null field references omit
    /// all text within the curly braces. If all field references are defined,
    /// all text within the curly braces are formatted without the curly braces.
    ///
    /// # Example
    ///
    /// ```
    /// use msica::{Field, Record};
    ///
    /// let record = Record::with_fields(
    ///     Some("this is [1] [2]{ without [3]}".to_owned()),
    ///     vec![Field::Integer(1), Field::String("example".to_owned()), Field::Null],
    /// );
    /// assert_eq!(record.format_text(), "this is 1 example");
    /// ```
    pub fn format_text(&self) -> String {
        unsafe {
            let mut value_len = 0u32;
            let value = CString::default();

            if ffi::MsiFormatRecord(
                MSIHANDLE::null(),
                self.into(),
                value.as_ptr() as ffi::LPSTR,
                &mut value_len as *mut u32,
            ) == ffi::ERROR_MORE_DATA
            {
                let mut value_len = value_len + 1u32;
                let mut value: Vec<u8> = vec![0; value_len as usize];

                ffi::MsiFormatRecord(
                    MSIHANDLE::null(),
                    self.into(),
                    value.as_mut_ptr() as ffi::LPSTR,
                    &mut value_len as *mut u32,
                );

                value.truncate(value_len as usize);
                return String::from_utf8(value).unwrap();
            }

            String::default()
        }
    }

    /// Gets a string field from a [`Record`].
    ///
    /// Field indices are 1-based, though you can get a template string from field 0.
    ///
    /// # Example
    ///
    /// ```
    /// use msica::{Field, Record};
    ///
    /// let record = Record::with_fields(
    ///     Some("this is [1] [2]".to_owned()),
    ///     vec![Field::Integer(1), Field::String("example".to_owned())],
    /// );
    /// assert_eq!(record.string(2), "example");
    /// ```
    pub fn string(&self, field: u32) -> String {
        unsafe {
            let mut value_len = 0u32;
            let value = CString::default();

            if ffi::MsiRecordGetString(
                self.into(),
                field,
                value.as_ptr() as ffi::LPSTR,
                &mut value_len as *mut u32,
            ) == ffi::ERROR_MORE_DATA
            {
                let mut value_len = value_len + 1u32;
                let mut value: Vec<u8> = vec![0; value_len as usize];

                ffi::MsiRecordGetString(
                    self.into(),
                    field,
                    value.as_mut_ptr() as ffi::LPSTR,
                    &mut value_len as *mut u32,
                );

                value.truncate(value_len as usize);
                return String::from_utf8(value).unwrap();
            }

            String::default()
        }
    }

    /// Sets a string field in a [`Record`].
    ///
    /// Field indices are 1-based, though you can set a template string in field 0.
    ///
    /// # Example
    ///
    /// ```
    /// use msica::{Field, Record};
    ///
    /// let mut record = Record::new(42);
    /// record.set_string(1, "example");
    /// assert_eq!(record.string(1), "example");
    /// ```
    pub fn set_string(&mut self, field: u32, value: &str) {
        unsafe {
            // TODO: Return result containing NulError if returned.
            let s = CString::new(value).unwrap();
            ffi::MsiRecordSetString(self.into(), field, s.as_ptr());
        }
    }

    /// Gets an integer field from a [`Record`].
    ///
    /// Field indices are 1-based.
    ///
    /// # Example
    ///
    /// ```
    /// use msica::{Field, Record};
    ///
    /// let record = Record::with_fields(
    ///     Some("this is [1] [2]".to_owned()),
    ///     vec![Field::Integer(1), Field::String("example".to_owned())],
    /// );
    /// assert_eq!(record.integer(1), 1);
    /// ```
    pub fn integer(&self, field: u32) -> i32 {
        unsafe { ffi::MsiRecordGetInteger(self.into(), field) }
    }

    /// Sets an integer field in a [`Record`].
    ///
    /// Field indices are 1-based.
    ///
    /// # Example
    ///
    /// ```
    /// use msica::{Field, Record};
    ///
    /// let mut record = Record::new(1);
    /// record.set_integer(1, 42);
    /// assert_eq!(record.integer(1), 42);
    /// ```
    pub fn set_integer(&mut self, field: u32, value: i32) {
        unsafe {
            ffi::MsiRecordSetInteger(self.into(), field, value);
        }
    }

    /// Gets whether a field is null in a [`Record`].
    ///
    /// Field indices are 1-based.
    ///
    /// # Example
    ///
    /// ```
    /// use msica::{Field, Record};
    ///
    /// let record = Record::new(1);
    /// assert_eq!(record.is_null(1), true);
    /// ```
    pub fn is_null(&self, field: u32) -> bool {
        unsafe { ffi::MsiRecordIsNull(self.into(), field).as_bool() }
    }
}

impl Into<MSIHANDLE> for Record {
    fn into(self) -> MSIHANDLE {
        MSIHANDLE(self.h.0)
    }
}

impl Into<MSIHANDLE> for &Record {
    fn into(self) -> MSIHANDLE {
        MSIHANDLE(self.h.0)
    }
}

impl Into<MSIHANDLE> for &mut Record {
    fn into(self) -> MSIHANDLE {
        MSIHANDLE(self.h.0)
    }
}

impl From<&str> for Record {
    fn from(s: &str) -> Self {
        unsafe {
            let h = ffi::MsiCreateRecord(0u32);
            // TODO: Return result containing NulError if returned.
            let s = CString::new(s).unwrap();
            ffi::MsiRecordSetString(h.into(), 0, s.as_ptr());

            Record { h: h.to_owned() }
        }
    }
}

impl From<String> for Record {
    fn from(s: String) -> Self {
        unsafe {
            let h = ffi::MsiCreateRecord(0u32);
            let s = CString::new(s).unwrap();
            ffi::MsiRecordSetString(h.into(), 0, s.as_ptr());

            Record { h: h.to_owned() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_count() {
        let record = Record::with_fields(
            None,
            vec![Field::Integer(1), Field::String("two".to_owned())],
        );
        assert_eq!(2, record.field_count());
    }

    #[test]
    fn format_text() {
        let record = Record::with_fields(
            Some("test [1] of [2]".to_owned()),
            vec![Field::Integer(2), Field::String("two".to_owned())],
        );
        assert_eq!("test 2 of two", record.format_text());
    }
}
