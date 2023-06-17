use crate::msg::MigrationListenerExecuteMsg;
use cosmwasm_std::{
    to_binary, Binary, CanonicalAddr, DepsMut, Response, StdError, StdResult, Storage, SubMsg,
    WasmMsg,
};
use schemars::_serde_json::to_string;

use crate::msg_types::ReplyError::OperationUnavailable;
use crate::state::{
    CanonicalContractInfo, ContractMode, MIGRATED_TO, MIGRATION_COMPLETE_EVENT_SUBSCRIBERS,
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
) -> StdResult<()> {
    if !allowed_contract_modes.contains(contract_mode) {
        Err(build_operation_unavailable_error(contract_mode, error_msg))
    } else {
        Ok(())
    }
}

pub fn register_to_notify_on_migration_complete(
    deps: DepsMut,
    contract_mode: ContractMode,
    address: String,
    code_hash: String,
) -> StdResult<Response> {
    check_contract_mode(vec![ContractMode::Running], &contract_mode, None)?;
    let validated = deps.api.addr_validate(address.as_str())?;
    add_migration_complete_event_subscriber(
        deps.storage,
        &deps.api.addr_canonicalize(validated.as_str())?,
        &code_hash,
    )?;
    Ok(Response::new())
}

pub fn broadcast_migration_complete_notification(
    deps: DepsMut,
    contract_mode: ContractMode,
    addresses: Vec<String>,
    code_hash: String,
    data: Option<Binary>,
) -> StdResult<Response> {
    check_contract_mode(vec![ContractMode::MigratedOut], &contract_mode, None)?;

    let migrated_to = MIGRATED_TO.load(deps.storage)?;
    let msg = to_binary(
        &MigrationListenerExecuteMsg::MigrationCompleteNotification {
            to: migrated_to.contract.into_humanized(deps.api)?,
            data,
        },
    )?;
    let sub_msgs = addresses
        .iter()
        .map(|address| {
            let contract_addr = deps.api.addr_validate(address)?.to_string();
            Ok(SubMsg::new(WasmMsg::Execute {
                msg: msg.clone(),
                contract_addr,
                code_hash: code_hash.clone(),
                funds: vec![],
            }))
        })
        .collect::<StdResult<Vec<SubMsg>>>()?;

    Ok(Response::new().add_submessages(sub_msgs))
}

pub fn add_migration_complete_event_subscriber(
    storage: &mut dyn Storage,
    address: &CanonicalAddr,
    code_hash: &str,
) -> StdResult<()> {
    if let Some(remaining_slots) = REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS.may_load(storage)? {
        if remaining_slots == 0 {
            return Err(StdError::generic_err(
                "No migration complete notification slots available",
            ));
        }
        REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS.save(storage, &(remaining_slots - 1))?
    }
    // todo: consider using a keyset if it does not increase gas usage
    let mut contracts = MIGRATION_COMPLETE_EVENT_SUBSCRIBERS
        .load(storage)
        .unwrap_or_default();
    let mut update = false;
    let new_contract = CanonicalContractInfo {
        address: address.clone(),
        code_hash: code_hash.to_string(),
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
    let notify_on_migration_contracts = MIGRATION_COMPLETE_EVENT_SUBSCRIBERS.may_load(storage)?;
    if let Some(mut notify_on_migration_contracts) = notify_on_migration_contracts {
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
    }
    Ok(())
}
