// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use msica::CustomActionResult::Succeed;
use msica::*;

const MSIDB_CUSTOM_ACTION_TYPE_DLL: i32 = 1;
const MSIDB_CUSTOM_ACTION_TYPE_CONTINUE: i32 = 64;
const MSIDB_CUSTOM_ACTION_TYPE_IN_SCRIPT: i32 = 1024;

#[no_mangle]
pub extern "C" fn TraceExampleCustomAction(session: Session) -> CustomActionResult {
    let trace_deferred_actions: Vec<&'static str> = vec!["InstallFiles"];

    let database = session.database();

    let custom_actions =
        database.open_view("SELECT `Action`, `Type`, `Source`, `Target` FROM `CustomAction`")?;
    custom_actions.execute(None)?;

    let sequence_table =
        database.open_view("SELECT `Action`, `Sequence` FROM `InstallExecuteSequence`")?;
    sequence_table.execute(None)?;

    let sequence_table_ordered = database.open_view(
        "SELECT `Action`, `Sequence` FROM `InstallExecuteSequence` ORDER BY `Sequence`",
    )?;
    sequence_table_ordered.execute(None)?;
    for action in sequence_table_ordered {
        let name = action.string_data(1)?;
        let sequence = action.integer_data(2).unwrap_or_default();

        if trace_deferred_actions.contains(&name.as_ref()) {
            let action = format!("{}Pre", name);
            session.set_property(action.as_ref(), Some(action.as_ref()))?;
            insert_row(&custom_actions, &sequence_table, action, sequence - 1)?;

            let action = format!("{}Post", name);
            session.set_property(action.as_ref(), Some(action.as_ref()))?;
            insert_row(&custom_actions, &sequence_table, action, sequence + 1)?;
        }
    }

    Succeed
}

#[no_mangle]
pub extern "C" fn TraceExampleCustomActionDeferred(session: Session) -> CustomActionResult {
    let data = session.property("CustomActionData")?;
    let record = Record::with_fields(Some("Running [1]"), vec![Field::StringData(data)])?;
    session.message(MessageType::Info, &record);

    Succeed
}

fn insert_row(
    custom_actions: &View,
    sequence_table: &View,
    name: String,
    sequence: i32,
) -> Result<()> {
    const TYPE: i32 = MSIDB_CUSTOM_ACTION_TYPE_DLL
        + MSIDB_CUSTOM_ACTION_TYPE_IN_SCRIPT
        + MSIDB_CUSTOM_ACTION_TYPE_CONTINUE;

    let custom_action = Record::with_fields(
        None,
        vec![
            Field::StringData(name.clone()),
            Field::IntegerData(TYPE),
            Field::StringData("TraceExample".to_owned()),
            Field::StringData("TraceExampleCustomActionDeferred".to_owned()),
        ],
    )?;
    custom_actions.modify(ModifyMode::InsertTemporary, &custom_action)?;

    let action = Record::with_fields(
        None,
        vec![Field::StringData(name), Field::IntegerData(sequence)],
    )?;
    sequence_table.modify(ModifyMode::InsertTemporary, &action)?;
    Ok(())
}
