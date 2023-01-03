use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};

use crate::msg::InstantiateMsg;
use crate::state::{OWNER, STATE, State};

pub fn instantiate(deps: DepsMut, msg: InstantiateMsg, info: MessageInfo) -> StdResult<Response> {
    let counter = msg.counter.unwrap_or_else(|| 0);
    let minimal_donation = msg.minimal_donation;
    STATE.save(
        deps.storage,
        &State {
            counter,
            minimal_donation,
        },
    )?;
    OWNER.save(deps.storage, &info.sender)?;
    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::msg::ValueResp;
    use crate::state::{STATE};

    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = STATE.load(deps.storage)?.counter;
        Ok(ValueResp { value })
    }

    pub fn increment(value: u64) -> StdResult<ValueResp> {
        Ok(ValueResp { value: value + 1 })
    }
}

pub mod exec {
    use cosmwasm_std::{BankMsg, Coin, DepsMut, Env, MessageInfo, Response, Uint128};

    use crate::error::ContractError;
    use crate::state::{OWNER, STATE};

    pub fn donate(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let mut state = STATE.load(deps.storage)?;

        if state.minimal_donation.amount.is_zero() ||
            info.funds.iter().any(|coin| {
                coin.denom == state.minimal_donation.denom && coin.amount >= state.minimal_donation.amount
            }) {
            state.counter += 1;
            STATE.save(deps.storage, &state)?;
        }

        let resp = Response::new()
            .add_attribute("action", "poke")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", state.counter.to_string());

        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if owner != info.sender {
            return Err(ContractError::Unauthorized { owner: owner.to_string() });
        }

        let balance = deps.querier.query_all_balances(&env.contract.address)?;

        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: balance,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }

    pub fn withdraw_to(deps: DepsMut, env: Env, info: MessageInfo, recipient: String, funds: Option<Vec<Coin>>) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if owner != info.sender {
            return Err(ContractError::Unauthorized { owner: owner.to_string() });
        }

        let mut balance = deps.querier.query_all_balances(&env.contract.address)?;

        if funds.is_some() {
            let funds = funds.unwrap();
            if !funds.is_empty() {
                for coin in &mut balance {
                    let limit = funds.
                        iter().
                        find(|c| c.denom == coin.denom).
                        map(|c| c.amount).
                        unwrap_or(Uint128::zero());

                    coin.amount = std::cmp::min(coin.amount, limit);
                }
            }
        }


        let bank_msg = BankMsg::Send {
            to_address: recipient,
            amount: balance,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }
}
