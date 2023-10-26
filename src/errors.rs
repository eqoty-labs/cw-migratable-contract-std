use cosmwasm_std::{StdError, SubMsgResult};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MigratableStdError {
    #[error("{0}")]
    // let thiserror implement From<StdError> for you
    Std(#[from] StdError),
    #[error("Migration complete event subscriber {0} has migrated. Request it to notify this contract of the new address. Error: {1}")]
    MigrationCompleteNotificationFailed(String, String),
}

pub fn humanize_migration_complete_notification_error(
    result: SubMsgResult,
) -> Result<(), MigratableStdError> {
    match result {
        SubMsgResult::Ok(_) => Ok(()),
        SubMsgResult::Err(error_msg) => {
            if error_msg.contains("failed to validate") {
                Err(MigratableStdError::MigrationCompleteNotificationFailed(
                    "someaddress".to_string(),
                    error_msg,
                ))
            } else {
                Err(MigratableStdError::Std(StdError::generic_err(error_msg)))
            }
        }
    }
}
