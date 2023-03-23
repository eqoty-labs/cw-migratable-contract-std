use cosmwasm_std::{to_binary, Binary, Deps, StdResult};

use crate::msg::MigratableQueryAnswer;
use crate::state::{MIGRATED_FROM, MIGRATED_TO};

/// Returns StdResult<Binary> displaying the Migrated to/from contract info
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `migrated_from` - if migrated_from is true query returns info about the contract it was migrated
/// from otherwise if returns info about the info the contract was migrated to
pub fn query_migrated_info(deps: Deps, migrated_from: bool) -> StdResult<Binary> {
    return match migrated_from {
        true => {
            let migrated_from = MIGRATED_FROM.may_load(deps.storage)?;
            match migrated_from {
                None => to_binary(&MigratableQueryAnswer::MigrationInfo(None)),
                Some(some_migrated_from) => to_binary(&MigratableQueryAnswer::MigrationInfo(Some(
                    some_migrated_from.contract,
                ))),
            }
        }
        false => {
            let migrated_to = MIGRATED_TO.may_load(deps.storage)?;
            match migrated_to {
                None => to_binary(&MigratableQueryAnswer::MigrationInfo(None)),
                Some(some_migrated_to) => to_binary(&MigratableQueryAnswer::MigrationInfo(Some(
                    some_migrated_to.contract,
                ))),
            }
        }
    };
}
