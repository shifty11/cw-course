use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};

use crate::msg::InstantiateMsg;
use crate::state::{COUNTER, MINIMAL_DONATION, OWNER};

pub fn instantiate(deps: DepsMut, msg: InstantiateMsg, info: MessageInfo) -> StdResult<Response> {
    COUNTER.save(deps.storage, &msg.counter.unwrap_or_else(|| 0))?;
    MINIMAL_DONATION.save(deps.storage, &msg.minimal_donation)?;
    OWNER.save(deps.storage, &info.sender)?;
    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::msg::ValueResp;
    use crate::state::COUNTER;

    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = COUNTER.load(deps.storage)?;
        Ok(ValueResp { value })
    }

    pub fn increment(value: u64) -> ValueResp {
        ValueResp { value: value + 1 }
    }
}

pub mod exec {
    use cosmwasm_std::{BankMsg, Coin, DepsMut, Env, MessageInfo, Response, Uint128};
    use crate::error::ContractError;

    use crate::state::{COUNTER, MINIMAL_DONATION, OWNER};

    pub fn donate(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let mut counter = COUNTER.load(deps.storage)?;
        let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;

        if minimal_donation.amount.is_zero() ||
            info.funds.iter().any(|coin| {
                coin.denom == minimal_donation.denom && coin.amount >= minimal_donation.amount
            }) {
            counter += 1;
            COUNTER.save(deps.storage, &counter)?;
        }

        let resp = Response::new()
            .add_attribute("action", "poke")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", counter.to_string());

        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if owner != info.sender {
            return Err(ContractError::Unauthorized {owner: owner.to_string()});
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

    pub fn withdraw_to(deps: DepsMut, env: Env, info: MessageInfo, recipient: String, funds: Option<Vec<Coin>>) -> Result<Response, ContractError>  {
        let owner = OWNER.load(deps.storage)?;
        if owner != info.sender {
            return Err(ContractError::Unauthorized {owner: owner.to_string()});
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

#[cfg(test)]
mod test {
    use std::borrow::{BorrowMut};

    use cosmwasm_std::{Addr, coin, coins, Empty};
    use cw_multi_test::{App, BasicApp, Contract, Executor};
    use cw_multi_test::ContractWrapper;

    use crate::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{ExecMsg, InstantiateMsg, QueryMsg, ValueResp};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    fn instantiate_contract(app: &mut BasicApp, contract_id: u64, counter: Option<u64>, minimal_donation: Option<u128>, sender: Option<Addr>) -> Addr {
        app.instantiate_contract(
            contract_id,
            sender.unwrap_or_else(|| Addr::unchecked("sender")),
            &InstantiateMsg { counter, minimal_donation: coin(minimal_donation.unwrap_or_else(|| 0), "atom") },
            &[],
            "Counting contract",
            None,
        )
            .unwrap()
    }

    #[test]
    fn instantiave_with_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = instantiate_contract(app.borrow_mut(), contract_id, Some(1234), None, None);

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 1234 });
    }

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = instantiate_contract(app.borrow_mut(), contract_id, None, None, None);

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 0 });
    }

    #[test]
    fn query_increment() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = instantiate_contract(app.borrow_mut(), contract_id, None, None, None);

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Increment { number: 5 })
            .unwrap();

        assert_eq!(resp, ValueResp { value: 6 });
    }

    #[test]
    fn donate() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = instantiate_contract(app.borrow_mut(), contract_id, None, None, None);

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &[],
        )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 1 });
    }

    #[test]
    fn donate_with_funds() {
        let sender = Addr::unchecked("sender");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, "atom"))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());

        let contract_addr = instantiate_contract(app.borrow_mut(), contract_id, None, Some(10), None);

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10, "atom"),
        )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 1 });
    }

    #[test]
    fn withdraw() {
        let owner = Addr::unchecked("owner");
        let sender = Addr::unchecked("sender");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, "atom"))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());

        let contract_addr = instantiate_contract(app.borrow_mut(), contract_id, None, Some(10), Some(owner.clone()));

        app.execute_contract(
            sender.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10, "atom"),
        )
            .unwrap();

        app.execute_contract(
            owner.clone(),
            contract_addr.clone(),
            &ExecMsg::Withdraw {},
            &[],
        )
            .unwrap();

        assert_eq!(
            app.wrap().query_all_balances(owner).unwrap(),
            coins(10, "atom")
        );
        assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
        assert_eq!(
            app.wrap().query_all_balances(contract_addr).unwrap(),
            vec![]
        );
    }

    #[test]
    fn withdraw_to() {
        let owner = Addr::unchecked("owner");
        let sender = Addr::unchecked("sender");
        let receiver = Addr::unchecked("receiver");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, "atom"))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());

        let contract_addr = instantiate_contract(app.borrow_mut(), contract_id, None, Some(10), Some(owner.clone()));

        app.execute_contract(
            sender.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10, "atom"),
        )
            .unwrap();

        app.execute_contract(
            owner.clone(),
            contract_addr.clone(),
            &ExecMsg::WithdrawTo {
                recipient: receiver.to_string(),
                funds: Some(coins(5, "atom")),
            },
            &[],
        )
            .unwrap();

        assert_eq!(app.wrap().query_all_balances(owner).unwrap(), vec![]);
        assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
        assert_eq!(
            app.wrap().query_all_balances(receiver).unwrap(),
            coins(5, "atom")
        );
        assert_eq!(
            app.wrap().query_all_balances(contract_addr).unwrap(),
            coins(5, "atom")
        );
    }

    #[test]
    fn unauthorized_withdraw() {
        let owner = Addr::unchecked("owner");
        let member = Addr::unchecked("member");

        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = instantiate_contract(app.borrow_mut(), contract_id, None, Some(10), Some(owner.clone()));

        let err = app
            .execute_contract(member, contract_addr, &ExecMsg::Withdraw {}, &[])
            .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {
                owner: owner.into()
            },
            err.downcast().unwrap()
        );
    }

    #[test]
    fn unauthorized_withdraw_to() {
        let owner = Addr::unchecked("owner");
        let member = Addr::unchecked("member");
        let recipient = Addr::unchecked("recipient");

        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = instantiate_contract(app.borrow_mut(), contract_id, None, Some(10), Some(owner.clone()));

        let err = app
            .execute_contract(member, contract_addr, &ExecMsg::WithdrawTo {recipient: recipient.to_string(), funds: None}, &[])
            .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {
                owner: owner.into()
            },
            err.downcast().unwrap()
        );
    }
}