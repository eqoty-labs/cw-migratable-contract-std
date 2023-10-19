use cosmwasm_std::{Api, CanonicalAddr, ContractInfo, StdResult};
use secret_toolkit::storage::Item;
use serde::{Deserialize, Serialize};

/// storage for list of contracts to notify when this contract has been migrated
pub static MIGRATION_COMPLETE_EVENT_SUBSCRIBERS: Item<Vec<CanonicalContractInfo>> =
    Item::new(b"ntifyOnMigrtd");
/// storage for an optional remaining number of contracts that can be registered to be notified of migration
pub static REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS: Item<u8> = Item::new(b"ntifyMigrtdSlts");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CanonicalContractInfo {
    pub address: CanonicalAddr,
    #[serde(default)]
    pub code_hash: String,
}

pub fn canonicalize(api: &dyn Api, value: &ContractInfo) -> StdResult<CanonicalContractInfo> {
    let c = CanonicalContractInfo {
        address: api.addr_canonicalize(value.address.as_str())?,
        code_hash: value.code_hash.clone(),
    };
    Ok(c)
}

impl CanonicalContractInfo {
    pub fn humanize(&self, api: &dyn Api) -> StdResult<ContractInfo> {
        let c = ContractInfo {
            address: api.addr_humanize(&self.address)?,
            code_hash: self.code_hash.clone(),
        };
        Ok(c)
    }

    pub fn into_humanized(self, api: &dyn Api) -> StdResult<ContractInfo> {
        let c = ContractInfo {
            address: api.addr_humanize(&self.address)?,
            code_hash: self.code_hash,
        };
        Ok(c)
    }
}
