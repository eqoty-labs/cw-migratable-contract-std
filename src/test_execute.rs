#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{
        to_binary, Addr, Binary, ContractInfo, StdError, StdResult, SubMsg, WasmMsg,
    };

    use crate::execute::{
        broadcast_migration_complete_notification, register_to_notify_on_migration_complete,
        update_migrated_subscriber,
    };
    use crate::msg::MigrationListenerExecuteMsg::MigrationCompleteNotification;
    use crate::state::{
        canonicalize, MIGRATION_COMPLETE_EVENT_SUBSCRIBERS,
        REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS,
    };

    #[test]
    fn register_to_notify_on_migration_complete_fails_with_when_no_slots_available() -> StdResult<()>
    {
        let mut deps = mock_dependencies();
        let receiver_address = "addr".to_string();
        let receiver_code_hash = "code_hash".to_string();
        REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS.save(deps.as_mut().storage, &1)?;
        let res = register_to_notify_on_migration_complete(
            deps.as_mut(),
            receiver_address.clone(),
            receiver_code_hash.clone(),
        );
        assert!(res.is_ok(), "execute failed");
        let res = register_to_notify_on_migration_complete(
            deps.as_mut(),
            receiver_address,
            receiver_code_hash,
        );
        assert!(res.is_err(), "execute didn't fail");
        assert_eq!(
            res.err().unwrap(),
            StdError::generic_err("No migration complete notification slots available")
        );
        Ok(())
    }

    #[test]
    fn register_to_notify_on_migration_complete_saves_contract() -> StdResult<()> {
        let mut deps = mock_dependencies();
        let receiver = &ContractInfo {
            address: Addr::unchecked("receiver_addr"),
            code_hash: "code_hash".to_string(),
        };
        register_to_notify_on_migration_complete(
            deps.as_mut(),
            receiver.address.to_string(),
            receiver.code_hash.to_string(),
        )?;
        let saved_contract = MIGRATION_COMPLETE_EVENT_SUBSCRIBERS.load(deps.as_ref().storage)?;
        assert_eq!(
            vec![canonicalize(deps.as_ref().api, receiver)?],
            saved_contract
        );
        Ok(())
    }

    #[test]
    fn update_migrated_subscriber_updates_storage() -> StdResult<()> {
        let mut deps = mock_dependencies();
        let subscriber_contract = canonicalize(
            deps.as_ref().api,
            &ContractInfo {
                address: Addr::unchecked("subscriber_addr"),
                code_hash: "subscriber_code_hash".to_string(),
            },
        )?;
        let migrated_subscriber_contract = canonicalize(
            deps.as_ref().api,
            &ContractInfo {
                address: Addr::unchecked("migrated_subscriber_addr"),
                code_hash: "migrated_subscriber_code_hash".to_string(),
            },
        )?;
        MIGRATION_COMPLETE_EVENT_SUBSCRIBERS
            .save(deps.as_mut().storage, &vec![subscriber_contract.clone()])?;

        update_migrated_subscriber(
            deps.as_mut().storage,
            &subscriber_contract.address,
            &migrated_subscriber_contract,
        )?;

        assert_eq!(
            vec![migrated_subscriber_contract],
            MIGRATION_COMPLETE_EVENT_SUBSCRIBERS.load(deps.as_ref().storage)?
        );

        Ok(())
    }

    #[test]
    fn update_migrated_subscriber_does_not_update_storage_when_sender_does_not_match(
    ) -> StdResult<()> {
        let mut deps = mock_dependencies();
        let subscriber_contract = canonicalize(
            deps.as_ref().api,
            &ContractInfo {
                address: Addr::unchecked("subscriber_addr"),
                code_hash: "subscriber_code_hash".to_string(),
            },
        )?;
        let random_contract = canonicalize(
            deps.as_ref().api,
            &ContractInfo {
                address: Addr::unchecked("random_addr"),
                code_hash: "random_code_hash".to_string(),
            },
        )?;
        let migrated_subscriber_contract = canonicalize(
            deps.as_ref().api,
            &ContractInfo {
                address: Addr::unchecked("migrated_subscriber_addr"),
                code_hash: "migrated_subscriber_code_hash".to_string(),
            },
        )?;
        MIGRATION_COMPLETE_EVENT_SUBSCRIBERS
            .save(deps.as_mut().storage, &vec![subscriber_contract.clone()])?;

        update_migrated_subscriber(
            deps.as_mut().storage,
            &random_contract.address,
            &migrated_subscriber_contract,
        )?;

        assert_eq!(
            vec![subscriber_contract],
            MIGRATION_COMPLETE_EVENT_SUBSCRIBERS.load(deps.as_ref().storage)?
        );

        Ok(())
    }

    #[test]
    fn broadcast_migration_complete_notification_creates_submsgs_for_all_specified_addresses(
    ) -> StdResult<()> {
        let deps = mock_dependencies();
        let migrated_to = &ContractInfo {
            address: Addr::unchecked("contract_addr"),
            code_hash: "contract_v2_code_hash".to_string(),
        };
        let broadcast_to_addresses = vec!["listener_a".to_string(), "listener_b".to_string()];
        let res = broadcast_migration_complete_notification(
            deps.as_ref(),
            migrated_to,
            broadcast_to_addresses.clone(),
            "listener_code_hash".to_string(),
            Some(Binary::from(b"payload")),
        )?;

        assert_eq!(
            broadcast_to_addresses
                .into_iter()
                .map(|addr| {
                    SubMsg::new(WasmMsg::Execute {
                        contract_addr: addr.to_string(),
                        code_hash: "listener_code_hash".to_string(),
                        msg: to_binary(&MigrationCompleteNotification {
                            to: migrated_to.clone(),
                            data: Some(Binary::from(b"payload")),
                        })
                        .unwrap(),
                        funds: vec![],
                    })
                })
                .collect::<Vec<SubMsg>>(),
            res.messages
        );

        Ok(())
    }
}
