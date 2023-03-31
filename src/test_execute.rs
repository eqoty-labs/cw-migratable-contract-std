#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Addr, ContractInfo, StdError, StdResult};
    use strum::IntoEnumIterator;

    use crate::execute::{
        build_operation_unavailable_error, register_to_notify_on_migration_complete,
        update_migrated_subscriber,
    };
    use crate::state::{
        canonicalize, ContractMode, CONTRACT_MODE, MIGRATION_COMPLETE_EVENT_SUBSCRIBERS,
        REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS,
    };

    #[test]
    fn register_to_notify_on_migration_complete_fails_with_when_no_slots_available() -> StdResult<()>
    {
        let mut deps = mock_dependencies();
        let receiver_address = "addr".to_string();
        let receiver_code_hash = "code_hash".to_string();
        CONTRACT_MODE.save(deps.as_mut().storage, &ContractMode::Running)?;
        REMAINING_MIGRATION_COMPLETE_EVENT_SUB_SLOTS.save(deps.as_mut().storage, &1)?;
        let mode = CONTRACT_MODE.load(deps.as_ref().storage)?;
        let res = register_to_notify_on_migration_complete(
            deps.as_mut(),
            mode,
            receiver_address.clone(),
            receiver_code_hash.clone(),
        );
        assert!(res.is_ok(), "execute failed");
        let mode = CONTRACT_MODE.load(deps.as_ref().storage)?;
        let res = register_to_notify_on_migration_complete(
            deps.as_mut(),
            mode,
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
        CONTRACT_MODE.save(deps.as_mut().storage, &ContractMode::Running)?;
        let receiver = &ContractInfo {
            address: Addr::unchecked("receiver_addr"),
            code_hash: "code_hash".to_string(),
        };
        let mode = CONTRACT_MODE.load(deps.as_ref().storage)?;
        register_to_notify_on_migration_complete(
            deps.as_mut(),
            mode,
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
    fn register_to_notify_on_migration_complete_fails_for_invalid_contract_modes() -> StdResult<()>
    {
        let mut deps = mock_dependencies();
        let invalid_modes: Vec<ContractMode> = ContractMode::iter()
            .filter(|m| m != &ContractMode::Running)
            .collect();
        let receiver = ContractInfo {
            address: Addr::unchecked("addr"),
            code_hash: "code_hash".to_string(),
        };
        for invalid_mode in invalid_modes {
            CONTRACT_MODE.save(deps.as_mut().storage, &invalid_mode)?;
            let mode = CONTRACT_MODE.load(deps.as_ref().storage)?;
            let res = register_to_notify_on_migration_complete(
                deps.as_mut(),
                mode,
                receiver.address.to_string(),
                receiver.code_hash.to_string(),
            );
            assert_eq!(
                res.err().unwrap(),
                build_operation_unavailable_error(&invalid_mode, None)
            );
        }
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
}
