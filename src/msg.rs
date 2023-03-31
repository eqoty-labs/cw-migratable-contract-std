use cosmwasm_std::{Binary, ContractInfo};
use schemars::JsonSchema;
use secret_toolkit::permit::Permit;
use serde::{Deserialize, Serialize};

use crate::msg_types::MigrateTo;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigratableExecuteMsg {
    /// Set migration secret (using entropy for randomness), and the address of the new contract
    Migrate {
        /// permit used to verify address executing migration is admin
        admin_permit: Permit,
        migrate_to: MigrateTo,
    },
    /// Sets a contract that should be notified when this contract completes the migration process
    SubscribeToOnMigrationCompleteEvent {
        address: String,
        code_hash: String,
        /// if true, the sender is requesting that the receiver of this message sends a reciprocal
        /// SubscribeToOnMigrationCompleteEvent message to the address (useful if the address is
        /// also a migratable contract)
        reciprocal_sub_requested: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrationListenerExecuteMsg {
    /// Upon a contract setting its ContractMode to MigratedOut. All contracts registered to be
    /// notified of a completed migration with RegisterToNotifyOnMigrationComplete should be sent
    /// a MigrationCompleteNotification message
    MigrationCompleteNotification {
        to: ContractInfo,
        // optional data to send
        data: Option<Binary>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigratableQueryMsg {
    MigratedFrom {},
    MigratedTo {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigratableQueryAnswer {
    MigrationInfo(Option<ContractInfo>),
}
