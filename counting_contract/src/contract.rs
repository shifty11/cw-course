
pub mod query {
    use crate::msg::ValueResp;

    pub fn value() -> ValueResp {
        ValueResp { value: 0 }
    }

    pub fn increment(value: u64) -> ValueResp {
        ValueResp { value: value + 1 }
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{Addr, Empty};

    use cw_multi_test::{App, Contract, Executor};
    use cw_multi_test::ContractWrapper;

    use crate::{execute, instantiate, query};
    use crate::msg::{QueryMsg, ValueResp};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &Empty {},
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

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

            let contract_addr = app
                .instantiate_contract(
                    contract_id,
                    Addr::unchecked("sender"),
                    &Empty {},
                    &[],
                    "Counting contract",
                    None,
                )
                .unwrap();

            let resp: ValueResp = app
                .wrap()
                .query_wasm_smart(contract_addr, &QueryMsg::Increment { number: 5 })
                .unwrap();

            assert_eq!(resp, ValueResp { value: 6 });
    }
}