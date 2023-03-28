#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{from_binary, Addr, ContractInfo, StdResult};

    use crate::msg::MigratableQueryAnswer;
    use crate::query::query_migrated_info;
    use crate::state::{
        canonicalize, MigratedFromState, MigratedToState, MIGRATED_FROM, MIGRATED_TO,
    };

    #[test]
    fn query_migrated_info_returns_saved_migrated_from_contract() -> StdResult<()> {
        let mut deps = mock_dependencies();
        let saved_migrated_from = MigratedFromState {
            contract: canonicalize(
                deps.as_ref().api,
                &ContractInfo {
                    address: Addr::unchecked("migrated_from_addr"),
                    code_hash: "migrated_from_code_hash".to_string(),
                },
            )?,
            migration_secret: Default::default(),
        };
        MIGRATED_FROM.save(deps.as_mut().storage, &saved_migrated_from)?;

        let queried_migrated_from: MigratableQueryAnswer =
            from_binary(&query_migrated_info(deps.as_ref(), true).unwrap())?;
        let MigratableQueryAnswer::MigrationInfo(queried_migrated_from) = queried_migrated_from;
        assert_eq!(
            Some(
                saved_migrated_from
                    .contract
                    .into_humanized(deps.as_ref().api)?
            ),
            queried_migrated_from
        );
        Ok(())
    }

    #[test]
    fn query_migrated_info_returns_saved_migrated_to_contract() -> StdResult<()> {
        let mut deps = mock_dependencies();
        let saved_migrated_to = MigratedToState {
            contract: canonicalize(
                deps.as_ref().api,
                &ContractInfo {
                    address: Addr::unchecked("migrated_to_addr"),
                    code_hash: "migrated_to_code_hash".to_string(),
                },
            )?,
            migration_secret: Default::default(),
        };
        MIGRATED_TO.save(deps.as_mut().storage, &saved_migrated_to)?;

        let queried_migrated_to: MigratableQueryAnswer =
            from_binary(&query_migrated_info(deps.as_ref(), false).unwrap())?;
        let MigratableQueryAnswer::MigrationInfo(queried_migrated_to) = queried_migrated_to;
        assert_eq!(
            Some(
                saved_migrated_to
                    .contract
                    .into_humanized(deps.as_ref().api)?
            ),
            queried_migrated_to
        );
        Ok(())
    }
}
