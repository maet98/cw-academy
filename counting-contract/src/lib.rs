use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

mod contract;
pub mod error;
pub mod msg;
#[cfg(test)]
pub mod multitest;
mod state;

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(_deps, _msg, _info)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;

    match msg {
        Value {} => to_binary(&contract::query::value(deps)?),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: msg::ExecMsg,
) -> Result<Response, ContractError> {
    use msg::ExecMsg::*;
    match _msg {
        Donate {} => contract::exec::donate(deps, _info).map_err(ContractError::from),
        Withdraw {} => contract::exec::withdraw(deps, _env, _info),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        msg::{ExecMsg, QueryMsg, ValueResp},
        multitest::CountingContract,
    };
    use cosmwasm_std::{coins, Addr, Coin, Empty};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    const ATOM: &str = "atom";

    #[test]
    fn query_value() {
        let mut app = App::default();
        let sender = Addr::unchecked("sender");

        let contract_id = app.store_code(counting_contract());

        let contract = CountingContract::instantiate(
            &mut app,
            contract_id,
            "Counting",
            &sender,
            Coin::new(10, ATOM),
        )
        .unwrap();

        let resp = contract.query_value(&app).unwrap();

        assert_eq!(resp, ValueResp { value: 0 });
    }

    #[test]
    fn donate() {
        let mut app = App::default();

        let sender = Addr::unchecked("sender");

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender.clone(),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10, ATOM),
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            sender.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &[],
        )
        .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();
        assert_eq!(resp, ValueResp { value: 0 });
    }

    #[test]
    fn donate_with_funds() {
        let sender = Addr::unchecked("sender");

        let mut app = AppBuilder::new().build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, ATOM))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender.clone(),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10, ATOM),
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            sender.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10, ATOM),
        )
        .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Value {})
            .unwrap();

        assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
        assert_eq!(
            app.wrap().query_all_balances(contract_addr).unwrap(),
            coins(10, ATOM)
        );
        assert_eq!(resp, ValueResp { value: 1 });
    }

    #[test]
    fn withdraw() {
        let owner = Addr::unchecked("owner");
        let sender1 = Addr::unchecked("sender1");
        let sender2 = Addr::unchecked("sender2");

        let mut app = AppBuilder::new().build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender1, coins(10, ATOM))
                .unwrap();
            router
                .bank
                .init_balance(storage, &sender2, coins(5, ATOM))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10, ATOM),
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            sender1.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10, ATOM),
        )
        .unwrap();

        app.execute_contract(
            sender2.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(5, ATOM),
        )
        .unwrap();

        assert_eq!(app.wrap().query_all_balances(sender1).unwrap(), vec![]);
        assert_eq!(app.wrap().query_all_balances(sender2).unwrap(), vec![]);

        app.execute_contract(
            owner.clone(),
            contract_addr.clone(),
            &ExecMsg::Withdraw {},
            &[],
        )
        .unwrap();

        assert_eq!(
            app.wrap()
                .query_all_balances(contract_addr.clone())
                .unwrap(),
            vec![]
        );
        assert_eq!(
            app.wrap().query_all_balances(owner.clone()).unwrap(),
            coins(15, ATOM)
        );
    }
}
