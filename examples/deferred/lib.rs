// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use msica::*;
const ERROR_SUCCESS: u32 = 0;

#[no_mangle]
pub extern "C" fn DeferredExampleCustomAction(h: MSIHANDLE) -> u32 {
    let session = Session::from(h);

    let database = session.database();
    let mut view = database
        .open_view("SELECT `Cardinal`, `Ordinal` FROM `DeferredExample` ORDER BY `Cardinal`");
    view.execute(None);
    for record in view.into_iter() {
        let data = format!(
            "{}\t{}",
            record.integer_data(1).unwrap(),
            record.string_data(2)
        );
        session.do_deferred_action("DeferredExampleCustomActionDeferred", &data);
    }
    ERROR_SUCCESS
}

#[no_mangle]
pub extern "C" fn DeferredExampleCustomActionDeferred(h: MSIHANDLE) -> u32 {
    let session = Session::from(h);

    // Process the custom action data passed by the immediate custom action.
    // This data is always made available in a property named "CustomActionData".
    let data = session.property("CustomActionData");
    let fields: Vec<&str> = data.split('\t').collect();
    let record = Record::with_fields(
        Some("Running the [2] ([1]) deferred custom action"),
        vec![
            Field::StringData(fields[0].to_string()),
            Field::StringData(fields[1].to_string()),
        ],
    );
    session.message(MessageType::Info, &record);
    ERROR_SUCCESS
}
