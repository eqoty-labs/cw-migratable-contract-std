use cosmwasm_std::{Binary, ContractInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigratableExecuteMsg {
    /// Sets a contract that should be notified when this contract completes the migration process
    SubscribeToMigrationCompleteEvent { address: String, code_hash: String },
    /// Triggers a MigrationCompleteNotification to be sent to the address specified. If this contract
    /// has been migrated out, otherwise it will return an error.
    ///
    /// Consider the case where you have a FactoryContract which only instantiates ChildContracts
    /// and only maintains a list of them which can become very large. In this scenario note that the
    /// only time a ChildContract needs to interact with the FactoryContract is when it migrates out.
    /// When a child Contract migrates out only then does it need to notify the FactoryContract of
    /// its new address. Now consider that the FactoryContract can also migrate. Instead of having
    /// each ChildContract call SubscribeToMigrationCompleteEvent on the FactoryContract (which could
    /// get very expensive with a large number of ChildContracts when the FactoryContract wants to
    /// migrate because it needs to notify every ChildContract before completing migration).
    /// The ChildContract can instead upon migration just query the FactoryContract address it has
    /// stored to check if it is the latest version before allowing a migration to continue.
    /// If the FactoryContract has migrated out the sender can call
    /// BroadcastMigrationCompleteNotification on the FactoryContract to notify the
    /// ChildContract of the new address. After that the ChildContract can proceed with its
    /// own migration.
    BroadcastMigrationCompleteNotification {
        /// addresses to send a MigrationCompleteNotification
        addresses: Vec<String>,
        /// the code for the addresses
        code_hash: String,
        data: Option<Binary>,
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
