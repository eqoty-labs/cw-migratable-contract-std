use cosmwasm_std::{
    CanonicalAddr, ContractInfo, DepsMut, MessageInfo, Response, StdError, StdResult,
};
use schemars::_serde_json::to_string;
use secret_toolkit::serialization::{Bincode2, Serde};
use secret_toolkit::storage::item::Item;

use crate::msg_types::ReplyError::OperationUnavailable;
use crate::state::{ContractMode, CONTRACT_MODE_KEY, NOTIFY_ON_MIGRATION_COMPLETE_KEY};

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
    info: MessageInfo,
    admin: CanonicalAddr,
    address: String,
    code_hash: String,
    contract_mode: Option<ContractMode>,
) -> StdResult<Response> {
    let contract_mode = if contract_mode.is_none() {
        Item::<ContractMode>::new(CONTRACT_MODE_KEY)
            .may_load(deps.storage)?
            .unwrap()
    } else {
        contract_mode.unwrap()
    };
    if let Some(contract_mode_error) =
        check_contract_mode(vec![ContractMode::Running], &contract_mode, None)
    {
        return Err(contract_mode_error);
    }
    let sender_raw = deps.api.addr_canonicalize(info.sender.as_str())?;
    if admin != sender_raw {
        return Err(StdError::generic_err(
            "This is an admin command and can only be run from the admin address",
        ));
    }
    let mut contracts = Item::<Vec<ContractInfo>>::new(NOTIFY_ON_MIGRATION_COMPLETE_KEY)
        .may_load(deps.storage)?
        .unwrap_or_default();
    let mut update = false;
    let new_contract = ContractInfo {
        address: deps.api.addr_validate(address.as_str())?,
        code_hash,
    };
    if !contracts.contains(&new_contract) {
        contracts.push(new_contract);
        update = true;
    }

    // only save if the list changed
    if update {
        deps.storage.set(
            NOTIFY_ON_MIGRATION_COMPLETE_KEY,
            &Bincode2::serialize(&contracts)?,
        );
    }
    Ok(Response::new())
}
