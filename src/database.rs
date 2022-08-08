// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::{MSIHANDLE, PMSIHANDLE};

/// The database for the current install session.
pub struct Database {
    h: PMSIHANDLE,
}

impl From<MSIHANDLE> for Database {
    fn from(h: MSIHANDLE) -> Self {
        Database { h: h.to_owned() }
    }
}
