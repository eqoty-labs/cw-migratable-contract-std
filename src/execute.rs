use cosmwasm_std::{CanonicalAddr, DepsMut, Response, StdError, StdResult, Storage};
use schemars::_serde_json::to_string;

use crate::msg_types::ReplyError::OperationUnavailable;
use crate::state::{
    CanonicalContractInfo, ContractMode, MIGRATION_COMPLETE_EVENT_SUBSCRIBERS,
    REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS,
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
    contract_mode: ContractMode,
    address: String,
    code_hash: String,
) -> StdResult<Response> {
    if let Some(contract_mode_error) =
        check_contract_mode(vec![ContractMode::Running], &contract_mode, None)
    {
        return Err(contract_mode_error);
    }
    if let Some(remaining_slots) =
        REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS.may_load(deps.storage)?
    {
        if remaining_slots == 0 {
            return Err(StdError::generic_err(
                "No migration complete notification slots available",
            ));
        }
        REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS.save(deps.storage, &(remaining_slots - 1))?
    }
    let validated = deps.api.addr_validate(address.as_str())?;
    add_migration_complete_event_subscriber(
        deps.storage,
        &deps.api.addr_canonicalize(validated.as_str())?,
        &code_hash,
    )?;
    Ok(Response::new())
}

pub fn add_migration_complete_event_subscriber(
    storage: &mut dyn Storage,
    address: &CanonicalAddr,
    code_hash: &String,
) -> StdResult<()> {
    let mut contracts = MIGRATION_COMPLETE_EVENT_SUBSCRIBERS
        .may_load(storage)?
        .unwrap_or_default();
    let mut update = false;
    let new_contract = CanonicalContractInfo {
        address: address.clone(),
        code_hash: code_hash.clone(),
    };
    if !contracts.contains(&new_contract) {
        contracts.push(new_contract);
        update = true;
    }

    // only save if the list changed
    if update {
        MIGRATION_COMPLETE_EVENT_SUBSCRIBERS.save(storage, &contracts)?;
    }
    Ok(())
}

pub fn update_migrated_subscriber(
    storage: &mut dyn Storage,
    raw_sender: &CanonicalAddr,
    raw_migrated_to: &CanonicalContractInfo,
) -> StdResult<()> {
    let mut notify_on_migration_contracts = MIGRATION_COMPLETE_EVENT_SUBSCRIBERS.load(storage)?;
    let mut update = false;

    for contract in notify_on_migration_contracts.iter_mut() {
        if &contract.address == raw_sender {
            contract.address = raw_migrated_to.address.clone();
            contract.code_hash = raw_migrated_to.code_hash.clone();
            update = true;
            break;
        }
    }
    if update {
        MIGRATION_COMPLETE_EVENT_SUBSCRIBERS.save(storage, &notify_on_migration_contracts)?;
    }
    Ok(())
}
