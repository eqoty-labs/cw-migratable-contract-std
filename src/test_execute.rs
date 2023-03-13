#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{Addr, Api, ContractInfo, StdError, StdResult};
    use strum::IntoEnumIterator;

    use crate::execute::{
        build_operation_unavailable_error, register_to_notify_on_migration_complete,
    };
    use crate::state::{ContractMode, CONTRACT_MODE, NOTIFY_ON_MIGRATION_COMPLETE};

    #[test]
    fn register_to_notify_on_migration_complete_fails_with_for_non_admin() -> StdResult<()> {
        let mut deps = mock_dependencies();
        let non_admin_info = mock_info("non_admin", &[]);
        let admin = deps.api.addr_canonicalize("admin").unwrap();
        let receiver_address = "addr".to_string();
        let receiver_code_hash = "code_hash".to_string();
        CONTRACT_MODE.save(deps.as_mut().storage, &ContractMode::Running)?;
        let res = register_to_notify_on_migration_complete(
            deps.as_mut(),
            non_admin_info,
            admin,
            receiver_address,
            receiver_code_hash,
            None,
        );
        assert!(res.is_err(), "execute didn't fail");
        assert_eq!(
            res.err().unwrap(),
            StdError::generic_err(
                "This is an admin command and can only be run from the admin address"
            )
        );
        Ok(())
    }

    #[test]
    fn register_to_notify_on_migration_complete_saves_contract() -> StdResult<()> {
        let mut deps = mock_dependencies();
        CONTRACT_MODE.save(deps.as_mut().storage, &ContractMode::Running)?;
        let admin_info = mock_info("admin", &[]);
        let admin = deps.api.addr_canonicalize("admin")?;
        let receiver = ContractInfo {
            address: Addr::unchecked("addr"),
            code_hash: "code_hash".to_string(),
        };
        register_to_notify_on_migration_complete(
            deps.as_mut(),
            admin_info,
            admin,
            receiver.address.to_string(),
            receiver.code_hash.to_string(),
            None,
        )?;
        let saved_contract: Vec<ContractInfo> =
            NOTIFY_ON_MIGRATION_COMPLETE.load(deps.as_ref().storage)?;
        assert_eq!(vec![receiver], saved_contract);
        Ok(())
    }

    #[test]
    fn register_to_notify_on_migration_complete_fails_for_invalid_contract_modes() -> StdResult<()>
    {
        let deps = mock_dependencies();
        let admin_info = mock_info("admin", &[]);
        let admin = deps.api.addr_canonicalize("admin")?;
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
            let res = register_to_notify_on_migration_complete(
                deps.as_mut(),
                admin_info.clone(),
                admin.clone(),
                receiver.address.to_string(),
                receiver.code_hash.to_string(),
                None,
            );
            assert_eq!(
                res.err().unwrap(),
                build_operation_unavailable_error(&invalid_mode, None)
            );
        }
        Ok(())
    }
}
