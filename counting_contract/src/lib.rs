#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult, to_binary};
use crate::error::ContractError;

mod contract;
pub mod msg;
mod state;
mod error;
#[cfg(any(test, feature = "tests"))]
pub mod multitest;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: msg::InstantiateMsg) -> StdResult<Response> {
    contract::instantiate(deps, msg, info)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;
    use contract::query;

    match msg {
        Value {} => to_binary(&query::value(deps)?),
        Increment { number } => to_binary(&query::increment(number)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: msg::ExecMsg) -> Result<Response, ContractError> {
    use contract::exec;
    use msg::ExecMsg::*;

    match msg {
        Donate {} => exec::donate(deps, env, info),
        Withdraw {} => exec::withdraw(deps, env, info),
        WithdrawTo { recipient, funds } => exec::withdraw_to(deps, env, info, recipient, funds),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    contract::migrate(deps)
}