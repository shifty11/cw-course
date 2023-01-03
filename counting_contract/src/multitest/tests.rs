use cosmwasm_std::{Addr, coin, coins};
use cw_multi_test::App;

use crate::error::ContractError;
use crate::msg::ValueResp;
use crate::state::{STATE, State};

use super::contract::CountingContract;
use counting_contract_0_1::multitest::contract::CountingContract as CountingContract_0_1;

const ATOM: &str = "atom";

#[test]
fn instantiate_with_value() {
    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        None,
        None,
        1234,
        None,
    ).unwrap();

    let resp: ValueResp = contract.query_value(&app).unwrap();

    assert_eq!(resp, ValueResp { value: 1234 });
}

#[test]
fn query_value() {
    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        None,
        None,
        None,
        None,
    ).unwrap();

    let resp: ValueResp = contract.query_value(&app).unwrap();

    assert_eq!(resp, ValueResp { value: 0 });
}

#[test]
fn query_increment() {
    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        None,
        None,
        None,
        None,
    ).unwrap();

    let resp: ValueResp = contract.query_increment(&app, 5).unwrap();

    assert_eq!(resp, ValueResp { value: 6 });
}

#[test]
fn donate() {
    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        None,
        None,
        None,
        None,
    ).unwrap();

    contract
        .donate(&mut app, &Addr::unchecked("sender"), &[])
        .unwrap();

    let resp: ValueResp = contract.query_value(&app).unwrap();

    assert_eq!(resp, ValueResp { value: 1 });
}

#[test]
fn donate_with_funds() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = CountingContract::app_with_funds(sender.clone(), 10);

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        None,
        10,
    ).unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 1 });
}

#[test]
fn withdraw() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = CountingContract::app_with_funds(sender.clone(), 10);

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        None,
        10,
    ).unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    contract
        .withdraw(&mut app, &owner.clone())
        .unwrap();

    assert_eq!(
        app.wrap().query_all_balances(owner).unwrap(),
        coins(10, "atom")
    );
    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        vec![]
    );
}

#[test]
fn withdraw_to() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");
    let receiver = Addr::unchecked("receiver");

    let mut app = CountingContract::app_with_funds(sender.clone(), 10);

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        None,
        10,
    ).unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    contract
        .withdraw_to(&mut app, &owner.clone(), &receiver, coins(5, ATOM))
        .unwrap();

    assert_eq!(app.wrap().query_all_balances(owner).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
    assert_eq!(
        app.wrap().query_all_balances(receiver).unwrap(),
        coins(5, "atom")
    );
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(5, "atom")
    );
}

#[test]
fn unauthorized_withdraw() {
    let owner = Addr::unchecked("owner");
    let member = Addr::unchecked("member");

    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        None,
        10,
    ).unwrap();

    let err = contract
        .withdraw(&mut app, &member)
        .unwrap_err();

    assert_eq!(
        err,
        ContractError::Unauthorized {
            owner: owner.into()
        },
    );
}

#[test]
fn unauthorized_withdraw_to() {
    let owner = Addr::unchecked("owner");
    let member = Addr::unchecked("member");
    let recipient = Addr::unchecked("recipient");

    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        None,
        10,
    ).unwrap();

    let err = contract
        .withdraw_to(&mut app, &member, &recipient, coins(5, ATOM))
        .unwrap_err();

    assert_eq!(
        err,
        ContractError::Unauthorized {
            owner: owner.into()
        },
    );
}

#[test]
fn migration() {
    let admin = Addr::unchecked("admin");
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });

    let old_code_id = CountingContract_0_1::store_code(&mut app);
    let new_code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract_0_1::instantiate(
        &mut app,
        old_code_id,
        owner.clone(),
        &admin,
        None,
        10,
    )
        .unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let contract =
        CountingContract::migrate(&mut app, contract.into(), new_code_id, &admin).unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 1 });

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            counter: 1,
            minimal_donation: coin(10, ATOM),
            owner: owner.clone(),
        }
    );
}

#[test]
fn migration_same_version() {
    let admin = Addr::unchecked("admin");
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        &admin,
        None,
        10,
    )
        .unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let contract =
        CountingContract::migrate(&mut app, contract.into(), code_id, &admin).unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 1 });

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            counter: 1,
            minimal_donation: coin(10, ATOM),
            owner: owner.clone(),
        }
    );
}