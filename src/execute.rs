use cosmwasm_std::{
    to_binary, CanonicalAddr, CosmosMsg, DepsMut, Response, StdError, StdResult, Storage, SubMsg,
    WasmMsg,
};
use schemars::_serde_json::to_string;

use crate::msg::MigratableExecuteMsg::SubscribeToOnMigrationCompleteEvent;
use crate::msg_types::ReplyError::OperationUnavailable;
use crate::state::{
    CanonicalContractInfo, ContractMode, NOTIFY_ON_MIGRATION_COMPLETE,
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
    contract_mode: ContractMode,
    address: String,
    code_hash: String,
    reciprocal_sub_requested: bool,
) -> StdResult<Response> {
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
        code_hash: code_hash.clone(),
    };
    if !contracts.contains(&new_contract) {
        contracts.push(new_contract);
        update = true;
    }

    // only save if the list changed
    if update {
        NOTIFY_ON_MIGRATION_COMPLETE.save(deps.storage, &contracts)?;
    }
    let mut sub_msgs = Vec::<SubMsg>::new();
    if reciprocal_sub_requested {
        sub_msgs.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address.clone(),
            code_hash: code_hash.clone(),
            msg: to_binary(&SubscribeToOnMigrationCompleteEvent {
                address,
                code_hash,
                reciprocal_sub_requested: false,
            })?,
            funds: vec![],
        })));
    }
    Ok(Response::new().add_submessages(sub_msgs))
}

pub fn update_migrated_subscriber(
    storage: &mut dyn Storage,
    raw_sender: &CanonicalAddr,
    raw_migrated_to: &CanonicalContractInfo,
) -> StdResult<()> {
    let mut notify_on_migration_contracts = NOTIFY_ON_MIGRATION_COMPLETE.load(storage)?;
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
        NOTIFY_ON_MIGRATION_COMPLETE.save(storage, &notify_on_migration_contracts)?;
    }
    Ok(())
}
