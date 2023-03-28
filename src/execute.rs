use cosmwasm_std::{DepsMut, Response, StdError, StdResult};
use schemars::_serde_json::to_string;

use crate::msg_types::ReplyError::OperationUnavailable;
use crate::state::{
    CanonicalContractInfo, ContractMode, CONTRACT_MODE, NOTIFY_ON_MIGRATION_COMPLETE,
    REMAINING_NOTIFY_ON_MIGRATION_COMPLETE_SLOTS,
};

pub fn build_operation_unavailable_error(
    contract_mode: &ContractMode,
    error_msg: Option<String>,
) -> StdError {
    StdError::generic_err(
        to_string(&OperationUnavailable {
            message: error_msg.unwrap_or(format!(
                "Not available in contact mode: {:?}.",
                contract_mode
            )),
        })
        .unwrap(),
    )
}

pub fn check_contract_mode(
    allowed_contract_modes: Vec<ContractMode>,
    contract_mode: &ContractMode,
    error_msg: Option<String>,
) -> Option<StdError> {
    return if !allowed_contract_modes.contains(contract_mode) {
        Some(build_operation_unavailable_error(contract_mode, error_msg))
    } else {
        None
    };
}

pub fn register_to_notify_on_migration_complete(
    deps: DepsMut,
    address: String,
    code_hash: String,
    contract_mode: Option<ContractMode>,
) -> StdResult<Response> {
    let contract_mode = if contract_mode.is_none() {
        CONTRACT_MODE.may_load(deps.storage)?.unwrap()
    } else {
        contract_mode.unwrap()
    };
    if let Some(contract_mode_error) =
        check_contract_mode(vec![ContractMode::Running], &contract_mode, None)
    {
        return Err(contract_mode_error);
    }
    if let Some(remaining_slots) =
        REMAINING_NOTIFY_ON_MIGRATION_COMPLETE_SLOTS.may_load(deps.storage)?
    {
        if remaining_slots == 0 {
            return Err(StdError::generic_err(
                "No migration complete notification slots available",
            ));
        }
        REMAINING_NOTIFY_ON_MIGRATION_COMPLETE_SLOTS.save(deps.storage, &(remaining_slots - 1))?
    }
    let mut contracts = NOTIFY_ON_MIGRATION_COMPLETE
        .may_load(deps.storage)?
        .unwrap_or_default();
    let mut update = false;
    let validated = deps.api.addr_validate(address.as_str())?;
    let new_contract = CanonicalContractInfo {
        address: deps.api.addr_canonicalize(validated.as_str())?,
        code_hash,
    };
    if !contracts.contains(&new_contract) {
        contracts.push(new_contract);
        update = true;
    }

    // only save if the list changed
    if update {
        NOTIFY_ON_MIGRATION_COMPLETE.save(deps.storage, &contracts)?;
    }
    Ok(Response::new())
}
