#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo,
    Response, StdResult, Timestamp,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetBalancesResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

const CONTRACT_NAME: &str = "crates.io:osmosis-revenue-transfer-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        channel_id: msg.channel_id.clone(),
        ibc_timeout: IbcTimeout::from(Timestamp::from_seconds(msg.ibc_timeout)),
        min_disbursal_amount: msg.min_disbursal_amount,
        memo: msg.memo.clone(),
        to_address: msg.to_address.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DisburseFunds { denom } => execute::disburse_funds(deps, env, denom),
    }
}

pub mod execute {
    use super::*;

    pub fn disburse_funds(
        deps: DepsMut,
        env: Env,
        denom: String,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let coin = match deps
            .querier
            .query_all_balances(env.contract.address)?
            .into_iter()
            .find(|c| c.denom == denom)
        {
            Some(coin) => coin,
            None => return Err(ContractError::DenomNotFound(denom)),
        };

        if coin.amount < state.min_disbursal_amount.into() {
            return Err(ContractError::InsufficientFunds {});
        }

        let msg = CosmosMsg::Ibc(IbcMsg::Transfer {
            channel_id: state.channel_id,
            to_address: state.to_address,
            amount: coin,
            timeout: state.ibc_timeout,
            memo: Some(state.memo),
        });

        Ok(Response::new().add_message(msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalances {} => to_json_binary(&query::balance(deps, env)?),
    }
}

pub mod query {
    use super::*;

    pub fn balance(deps: Deps, env: Env) -> StdResult<GetBalancesResponse> {
        let coins = deps.querier.query_all_balances(env.contract.address)?;

        Ok(GetBalancesResponse { balances: coins })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies_with_balance, mock_env};
    use cosmwasm_std::{coins, from_json, Addr, Coin, Uint128};

    #[test]
    fn disburse_funds() {
        let mut deps = mock_dependencies_with_balance(&[Coin {
            amount: Uint128::new(2000),
            denom: "denom".to_string(),
        }]);

        let msg = InstantiateMsg {
            min_disbursal_amount: 0,
            channel_id: "channel-0".to_string(),
            ibc_timeout: 1000,
            memo: "memo".to_string(),
            to_address: "to_address".to_string(),
        };
        let info = message_info(&Addr::unchecked("123"), &coins(2000, "denom"));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let msg = ExecuteMsg::DisburseFunds {
            denom: "denom".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    #[test]
    fn balances() {
        let mut deps = mock_dependencies_with_balance(&[Coin {
            amount: Uint128::new(2000),
            denom: "denom".to_string(),
        }]);

        let msg = InstantiateMsg {
            min_disbursal_amount: 0,
            channel_id: "channel-0".to_string(),
            ibc_timeout: 1000,
            memo: "memo".to_string(),
            to_address: "to_address".to_string(),
        };
        let info = message_info(&Addr::unchecked("123"), &coins(2000, "denom"));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let msg = QueryMsg::GetBalances {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let value: GetBalancesResponse = from_json(&res).unwrap();
        assert_eq!(
            vec![Coin {
                amount: Uint128::new(2000),
                denom: "denom".to_string(),
            }],
            value.balances
        );
    }
}
