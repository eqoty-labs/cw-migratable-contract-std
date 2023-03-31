#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{
        from_binary, Addr, Coin, ContractInfo, CosmosMsg, StdError, StdResult, WasmMsg,
    };
    use strum::IntoEnumIterator;

    use crate::execute::{
        build_operation_unavailable_error, register_to_notify_on_migration_complete,
        update_migrated_subscriber,
    };
    use crate::msg::MigratableExecuteMsg::SubscribeToOnMigrationCompleteEvent;
    use crate::state::{
        canonicalize, ContractMode, CONTRACT_MODE, NOTIFY_ON_MIGRATION_COMPLETE,
        REMAINING_NOTIFY_ON_MIGRATION_COMPLETE_SLOTS,
    };

    #[test]
    fn register_to_notify_on_migration_complete_fails_with_when_no_slots_available() -> StdResult<()>
    {
        let mut deps = mock_dependencies();
        let receiver_address = "addr".to_string();
        let receiver_code_hash = "code_hash".to_string();
        CONTRACT_MODE.save(deps.as_mut().storage, &ContractMode::Running)?;
        REMAINING_NOTIFY_ON_MIGRATION_COMPLETE_SLOTS.save(deps.as_mut().storage, &1)?;
        let mode = CONTRACT_MODE.load(deps.as_ref().storage)?;
        let res = register_to_notify_on_migration_complete(
            deps.as_mut(),
            mode,
            receiver_address.clone(),
            receiver_code_hash.clone(),
            false,
        );
        assert!(res.is_ok(), "execute failed");
        let mode = CONTRACT_MODE.load(deps.as_ref().storage)?;
        let res = register_to_notify_on_migration_complete(
            deps.as_mut(),
            mode,
            receiver_address,
            receiver_code_hash,
            false,
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
            false,
        )?;
        let saved_contract = NOTIFY_ON_MIGRATION_COMPLETE.load(deps.as_ref().storage)?;
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
                false,
            );
            assert_eq!(
                res.err().unwrap(),
                build_operation_unavailable_error(&invalid_mode, None)
            );
        }
        Ok(())
    }

    #[test]
    fn register_to_notify_on_migration_complete_creates_reciprocal_subscribe_submsg(
    ) -> StdResult<()> {
        let mut deps = mock_dependencies();
        CONTRACT_MODE.save(deps.as_mut().storage, &ContractMode::Running)?;
        let receiver = &ContractInfo {
            address: Addr::unchecked("receiver_addr"),
            code_hash: "code_hash".to_string(),
        };
        let mode = CONTRACT_MODE.load(deps.as_ref().storage)?;
        let res = register_to_notify_on_migration_complete(
            deps.as_mut(),
            mode,
            receiver.address.to_string(),
            receiver.code_hash.to_string(),
            true,
        )?;

        match &res.messages[0].msg {
            CosmosMsg::Wasm(msg) => match msg {
                WasmMsg::Execute {
                    contract_addr,
                    code_hash,
                    msg,
                    funds,
                    ..
                } => {
                    assert_eq!(
                        SubscribeToOnMigrationCompleteEvent {
                            address: receiver.address.to_string(),
                            code_hash: receiver.code_hash.to_string(),
                            reciprocal_sub_requested: false,
                        },
                        from_binary(msg)?
                    );
                    assert_eq!(contract_addr, &receiver.address);
                    assert_eq!(code_hash, &receiver.code_hash);
                    assert_eq!(&Vec::<Coin>::new(), funds);
                }
                _ => panic!("unexpected"),
            },
            _ => panic!("unexpected"),
        }
        Ok(())
    }

    #[test]
    fn register_to_notify_on_migration_complete_creates_no_submsg_when_not_requested(
    ) -> StdResult<()> {
        let mut deps = mock_dependencies();
        CONTRACT_MODE.save(deps.as_mut().storage, &ContractMode::Running)?;
        let receiver = &ContractInfo {
            address: Addr::unchecked("receiver_addr"),
            code_hash: "code_hash".to_string(),
        };
        let mode = CONTRACT_MODE.load(deps.as_ref().storage)?;
        let res = register_to_notify_on_migration_complete(
            deps.as_mut(),
            mode,
            receiver.address.to_string(),
            receiver.code_hash.to_string(),
            false,
        )?;

        assert_eq!(0, res.messages.len());
        Ok(())
    }

}
