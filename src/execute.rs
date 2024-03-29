use cosmwasm_std::{
    to_binary, Binary, CanonicalAddr, ContractInfo, CosmosMsg, Deps, DepsMut, ReplyOn, Response,
    StdError, StdResult, Storage, SubMsg, WasmMsg,
};

use crate::msg::MigrationListenerExecuteMsg;
use crate::state::{
    CanonicalContractInfo, MIGRATION_COMPLETE_EVENT_SUBSCRIBERS,
    REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS,
};

pub fn register_to_notify_on_migration_complete(
    deps: DepsMut,
    address: String,
    code_hash: String,
) -> StdResult<Response> {
    let validated = deps.api.addr_validate(address.as_str())?;
    add_migration_complete_event_subscriber(
        deps.storage,
        &deps.api.addr_canonicalize(validated.as_str())?,
        &code_hash,
    )?;
    Ok(Response::new())
}

pub fn create_broadcast_migration_complete_notification_msgs(
    deps: Deps,
    reply_on: ReplyOn,
    reply_id: u64,
    migrated_to: &ContractInfo,
    notification_recipients: Vec<ContractInfo>,
    data: Option<Binary>,
) -> StdResult<Vec<SubMsg>> {
    let msg = to_binary(
        &MigrationListenerExecuteMsg::MigrationCompleteNotification {
            to: migrated_to.clone(),
            data,
        },
    )?;
    let sub_msgs = notification_recipients
        .into_iter()
        .map(|contract| {
            let contract_addr = deps
                .api
                .addr_validate(contract.address.as_str())?
                .to_string();
            Ok(SubMsg {
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    msg: msg.clone(),
                    contract_addr,
                    code_hash: contract.code_hash,
                    funds: vec![],
                }),
                id: reply_id,
                reply_on: reply_on.clone(),
                gas_limit: None,
            })
        })
        .collect::<StdResult<Vec<SubMsg>>>()?;

    Ok(sub_msgs)
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
