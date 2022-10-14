// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::ffi;
use std::{ffi::CString, fmt::Display};

/// A field in a [`Record`].
pub enum Field {
    /// A string field in a [`Record`].
    StringData(String),

    /// An integer field in a [`Record`].
    IntegerData(i32),

    /// A null field in a [`Record`].
    Null,
}

/// A collection of [`Field`] containing strings, integers, and byte streams.
#[derive(Debug)]
pub struct Record {
    pub(crate) h: ffi::PMSIHANDLE,
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
    ///     Some("this is [1] [2]"),
    ///     vec![Field::IntegerData(1), Field::StringData("example".to_owned())]);
    /// assert_eq!(record.field_count(), 2);
    /// ```
    pub fn with_fields(text: Option<&str>, fields: Vec<Field>) -> Self {
        unsafe {
            let h = ffi::MsiCreateRecord(fields.len() as u32);
            let record = Record { h: h.to_owned() };

            if let Some(text) = text {
                record.set_string_data(0, Some(text));
            }

            for (i, field) in fields.iter().enumerate() {
                let i: u32 = i.try_into().unwrap();
                match field {
                    Field::StringData(data) => record.set_string_data(i + 1, Some(data)),
                    Field::IntegerData(data) => record.set_integer_data(i + 1, *data),
                    Field::Null => {}
                }
            }

            record
        }
    }

    /// Gets the count of fields in the record.
    pub fn field_count(&self) -> u32 {
        unsafe { ffi::MsiRecordGetFieldCount(*self.h) }
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
    ///     Some("this is [1] [2]{ without [3]}"),
    ///     vec![Field::IntegerData(1), Field::StringData("example".to_owned()), Field::Null],
    /// );
    /// assert_eq!(record.format_text(), "this is 1 example");
    /// ```
    pub fn format_text(&self) -> String {
        unsafe {
            let mut value_len = 0u32;
            let value = CString::default();

            if ffi::MsiFormatRecord(
                ffi::MSIHANDLE::null(),
                *self.h,
                value.as_ptr() as ffi::LPSTR,
                &mut value_len as *mut u32,
            ) == ffi::ERROR_MORE_DATA
            {
                let mut value_len = value_len + 1u32;
                let mut value: Vec<u8> = vec![0; value_len as usize];

                ffi::MsiFormatRecord(
                    ffi::MSIHANDLE::null(),
                    *self.h,
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
    ///     Some("this is [1] [2]"),
    ///     vec![Field::IntegerData(1), Field::StringData("example".to_owned())],
    /// );
    /// assert_eq!(record.string_data(2), "example");
    /// ```
    pub fn string_data(&self, field: u32) -> String {
        unsafe {
            let mut value_len = 0u32;
            let value = CString::default();

            if ffi::MsiRecordGetString(
                *self.h,
                field,
                value.as_ptr() as ffi::LPSTR,
                &mut value_len as *mut u32,
            ) == ffi::ERROR_MORE_DATA
            {
                let mut value_len = value_len + 1u32;
                let mut value: Vec<u8> = vec![0; value_len as usize];

                ffi::MsiRecordGetString(
                    *self.h,
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

    /// Sets a string field in a [`Record`]. Pass `None` to clear the field.
    ///
    /// Field indices are 1-based, though you can set a template string in field 0.
    ///
    /// # Example
    ///
    /// ```
    /// use msica::{Field, Record};
    ///
    /// let mut record = Record::new(1);
    /// record.set_string_data(1, Some("example"));
    /// assert_eq!(record.string_data(1), "example");
    /// ```
    pub fn set_string_data(&self, field: u32, value: Option<&str>) {
        unsafe {
            // TODO: Return result containing NulError if returned.
            let value = match value {
                Some(s) => CString::new(s).unwrap(),
                None => CString::default(),
            };
            ffi::MsiRecordSetString(*self.h, field, value.as_ptr());
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
    ///     Some("this is [1] [2]"),
    ///     vec![Field::IntegerData(1), Field::StringData("example".to_owned())],
    /// );
    /// assert_eq!(record.integer_data(1), Some(1));
    /// ```
    pub fn integer_data(&self, field: u32) -> Option<i32> {
        unsafe {
            match ffi::MsiRecordGetInteger(*self.h, field) {
                i if i == ffi::MSI_NULL_INTEGER => None,
                i => Some(i),
            }
        }
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
    /// record.set_integer_data(1, 42);
    /// assert_eq!(record.integer_data(1), Some(42));
    /// ```
    pub fn set_integer_data(&self, field: u32, value: i32) {
        unsafe {
            ffi::MsiRecordSetInteger(*self.h, field, value);
        }
    }

    /// Reads bytes from a record field that contains stream data.
    ///
    /// Field indices are 1-based.
    #[allow(unused_variables)]
    pub fn stream_data(&self, field: u32) -> Vec<u8> {
        todo!()
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
        unsafe { ffi::MsiRecordIsNull(*self.h, field).as_bool() }
    }

    pub(crate) fn from_handle(h: ffi::MSIHANDLE) -> Self {
        Record { h: h.to_owned() }
    }
}

impl<T> From<T> for Record
where
    T: AsRef<str>,
{
    fn from(s: T) -> Self {
        unsafe {
            let h = ffi::MsiCreateRecord(0u32);
            // TODO: Return result containing NulError if returned.
            let s = CString::new(s.as_ref()).unwrap();
            ffi::MsiRecordSetString(h, 0, s.as_ptr());

            Record { h: h.to_owned() }
        }
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.format_text();
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let record = Record::from("test");
        assert_eq!(record.string_data(0), "test");
    }

    #[test]
    fn from_string() {
        let record = Record::from("test".to_owned());
        assert_eq!(record.string_data(0), "test");
    }

    #[test]
    fn set_string_data_null() {
        let record = Record::with_fields(None, vec![Field::StringData("test".to_owned())]);
        assert_eq!(record.string_data(1), "test");

        record.set_string_data(1, None);
        assert!(record.is_null(1));
        assert_eq!(record.string_data(1), "");
    }

    #[test]
    fn integer_data_from_string() {
        let record = Record::with_fields(None, vec![Field::StringData("test".to_owned())]);
        assert_eq!(record.integer_data(1), None);
    }
}
