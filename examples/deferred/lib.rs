// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use msica::*;
const ERROR_SUCCESS: u32 = 0;

#[no_mangle]
pub extern "C" fn DeferredExampleCustomAction(h: MSIHANDLE) -> u32 {
    let session = Session::from(h);

    // Simulate reading data from a custom table.
    for i in 0..5 {
        session.do_deferred_action("DeferredExampleCustomActionDeferred", &i.to_string())
    }
    ERROR_SUCCESS
}

#[no_mangle]
pub extern "C" fn DeferredExampleCustomActionDeferred(h: MSIHANDLE) -> u32 {
    let session = Session::from(h);

    // Process the custom action data passed by the immediate custom action.
    // This data is always made available in a property named "CustomActionData".
    let data = session.property("CustomActionData");
    let record = Record::with_fields(
        Some("Running deferred custom action [1]"),
        vec![Field::StringData(data)],
    );
    session.message(MessageType::Info, &record);
    ERROR_SUCCESS
}
