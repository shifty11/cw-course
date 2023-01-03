use cosmwasm_std::{Addr, Coin, coin, coins, StdResult};
use cw_multi_test::{App, BasicApp, Executor};
use cw_multi_test::ContractWrapper;

use crate::{execute, instantiate, query};
use crate::error::ContractError;
use crate::msg::{ExecMsg, InstantiateMsg, QueryMsg, ValueResp};

pub struct CountingContract(Addr);

impl CountingContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn app_with_funds(sender: impl Into<Option<Addr>>, amount: impl Into<Option<u128>>) -> BasicApp {
        App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &sender.into().unwrap_or_else(|| Addr::unchecked("sender")),
                    coins(amount.into().unwrap_or_else(|| 0), "atom"))
                .unwrap();
        })
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate<'a>(
        app: &mut App,
        code_id: u64,
        sender: impl Into<Option<Addr>>,
        admin: impl Into<Option<&'a Addr>>,
        counter: impl Into<Option<u64>>,
        minimal_donation: impl Into<Option<u128>>,
    ) -> StdResult<Self> {
        let counter = counter.into().unwrap_or_default();
        let admin = admin.into().map(Addr::to_string);

        app.instantiate_contract(
            code_id,
            sender.into().unwrap_or_else(|| Addr::unchecked("sender")),
            &InstantiateMsg {
                counter: Some(counter),
                minimal_donation: coin(minimal_donation.into().unwrap_or_else(|| 0), "atom"),
            },
            &[],
            "Counting contract",
            admin,
        )
            .map(CountingContract)
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn donate(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Donate {}, funds)
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn withdraw(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Withdraw {}, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn withdraw_to(
        &self,
        app: &mut App,
        sender: &Addr,
        receiver: &Addr,
        funds: impl Into<Option<Vec<Coin>>>,
    ) -> Result<(), ContractError> {
        let funds = funds.into().unwrap_or_default();
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::WithdrawTo {
                recipient: receiver.to_string(),
                funds: Some(funds),
            },
            &[],
        )
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn query_value(&self, app: &App) -> StdResult<ValueResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Value {})
    }

    #[track_caller]
    pub fn query_increment(&self, app: &App, number: u64) -> StdResult<ValueResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Increment { number })
    }
}

impl From<CountingContract> for Addr {
    fn from(contract: CountingContract) -> Self {
        contract.0
    }
}