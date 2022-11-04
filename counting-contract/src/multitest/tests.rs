use cosmwasm_std::{coins, Addr, Coin, Empty};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

use crate::{
    error::ContractError,
    execute, instantiate,
    msg::{QueryMsg, ValueResp},
    multitest::CountingContract,
    query,
};

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

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        "Counting contract",
        &sender,
        Coin::new(10, ATOM),
    )
    .unwrap();

    contract.donate(&mut app, &sender, &[]).unwrap();

    let resp: ValueResp = contract.query_value(&app).unwrap();
    assert_eq!(resp.value, 0);
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

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        "Counting contract",
        &sender,
        Coin::new(10, ATOM),
    )
    .unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(10, ATOM)
    );
    assert_eq!(resp.value, 1);
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

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        "Counting contract",
        &owner,
        Coin::new(10, ATOM),
    )
    .unwrap();

    contract
        .donate(&mut app, &sender1.clone(), &coins(10, ATOM))
        .unwrap();
    contract
        .donate(&mut app, &sender2.clone(), &coins(5, ATOM))
        .unwrap();

    assert_eq!(app.wrap().query_all_balances(sender1).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(sender2).unwrap(), vec![]);

    contract.withdraw(&mut app, &owner.clone(), &[]).unwrap();

    assert_eq!(
        app.wrap()
            .query_all_balances(contract.addr().clone())
            .unwrap(),
        vec![]
    );
    assert_eq!(
        app.wrap().query_all_balances(owner.clone()).unwrap(),
        coins(15, ATOM)
    );
}

#[test]
fn unathorized_withdraw() {
    let owner = Addr::unchecked("owner");
    let member = Addr::unchecked("member");

    let mut app = App::default();

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        "Counting contract",
        &owner,
        Coin::new(10, ATOM),
    )
    .unwrap();

    let err = contract.withdraw(&mut app, &member, &[]).unwrap_err();

    assert_eq!(
        err,
        ContractError::Unauthorized {
            owner: owner.into()
        }
    );
}
