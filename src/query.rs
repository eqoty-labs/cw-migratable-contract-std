use cosmwasm_std::{to_binary, Binary, Deps, StdResult};
use serde::{Deserialize, Serialize};

use crate::msg::MigratableQueryAnswer;
use crate::state::{MIGRATED_FROM, MIGRATED_TO};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[repr(u8)]
pub enum MigrationDirection {
    From = 1,
    To = 2,
}
/// Returns StdResult<Binary> displaying the Migrated to/from contract info
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `direction` - specifies which migration direction to query a contract about To or From
pub fn query_migrated_info(deps: Deps, direction: MigrationDirection) -> StdResult<Binary> {
    return match direction {
        MigrationDirection::From => {
            let migrated_from = MIGRATED_FROM.may_load(deps.storage)?;
            match migrated_from {
                None => to_binary(&MigratableQueryAnswer::MigrationInfo(None)),
                Some(some_migrated_from) => to_binary(&MigratableQueryAnswer::MigrationInfo(Some(
                    some_migrated_from.contract.into_humanized(deps.api)?,
                ))),
            }
        }
        MigrationDirection::To => {
            let migrated_to = MIGRATED_TO.may_load(deps.storage)?;
            match migrated_to {
                None => to_binary(&MigratableQueryAnswer::MigrationInfo(None)),
                Some(some_migrated_to) => to_binary(&MigratableQueryAnswer::MigrationInfo(Some(
                    some_migrated_to.contract.into_humanized(deps.api)?,
                ))),
            }
        }
    };
}
