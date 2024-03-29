// Copyright 2024 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use msica::prelude::*;

#[no_mangle]
pub extern "C" fn DeferredExampleCustomAction(session: Session) -> CustomActionResult {
    let database = session.database();
    let view = database
        .open_view("SELECT `Cardinal`, `Ordinal` FROM `DeferredExample` ORDER BY `Cardinal`")?;

    // Add another row.
    view.modify(
        ModifyMode::InsertTemporary,
        &Record::with_fields(
            None,
            vec![
                Field::IntegerData(100),
                Field::StringData("last".to_string()),
            ],
        )?,
    )?;

    // Schedule custom actions for each row.
    view.execute(None)?;
    for record in view {
        let data = format!(
            "{}\t{}",
            record.integer_data(1).unwrap_or(0),
            record.string_data(2)?,
        );
        session.do_deferred_action("DeferredExampleCustomActionDeferred", &data)?;
    }
    Success
}

#[no_mangle]
pub extern "C" fn DeferredExampleCustomActionDeferred(session: Session) -> CustomActionResult {
    // Process the custom action data passed by the immediate custom action.
    // This data is always made available in a property named "CustomActionData".
    let data = session.property("CustomActionData")?;
    let fields: Vec<&str> = data.split('\t').collect();
    let record = Record::with_fields(
        Some("Running the [2] ([1]) deferred custom action"),
        vec![
            Field::StringData(fields[0].to_string()),
            Field::StringData(fields[1].to_string()),
        ],
    )?;
    session.message(MessageType::Info, &record);
    Success
}
