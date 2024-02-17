// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use msica::prelude::*;

#[no_mangle]
pub extern "C" fn SkipExampleCustomAction(session: Session) -> CustomActionResult {
    let deferred = session.mode(RunMode::Scheduled);
    match deferred {
        false => {
            let data = session.property("SKIP")?;
            if data == "1" {
                return Skip;
            }
            session.do_deferred_action("SkipExampleCustomActionDeferred", data.as_str())?;
        }
        true => {
            let data = session.property("CustomActionData")?;
            if data.is_empty() {
                return Success;
            }

            // Unnecessarily parsing the string demonstrates using ? for any possible error.
            let data = data.parse::<u32>()?;
            if data == 2 {
                return Skip;
            }
        }
    }
    Success
}
