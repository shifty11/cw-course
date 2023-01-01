use cosmwasm_std::{Binary, Deps, DepsMut, entry_point, Env, MessageInfo, Response, StdResult, to_binary};
use crate::error::ContractError;

mod contract;
pub mod msg;
mod state;
mod error;

#[entry_point]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: msg::InstantiateMsg) -> StdResult<Response> {
    contract::instantiate(deps, msg, info)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;
    use contract::query;

    match msg {
        Value {} => to_binary(&query::value(deps)?),
        Increment { number } => to_binary(&query::increment(number)?),
    }
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: msg::ExecMsg) -> Result<Response, ContractError> {
    use contract::exec;
    use msg::ExecMsg::*;

    match msg {
        Donate {} => exec::donate(deps, info),
        Withdraw {} => exec::withdraw(deps, env, info),
        WithdrawTo { recipient, funds } => exec::withdraw_to(deps, env, info, recipient, funds),
    }
}