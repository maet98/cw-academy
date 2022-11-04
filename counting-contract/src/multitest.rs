use cosmwasm_std::{Addr, Coin, StdError, StdResult};
use cw_multi_test::{App, Executor};

mod tests;

use crate::{
    error::ContractError,
    msg::{ExecMsg, InstantiateMsg, QueryMsg, ValueResp},
};

pub struct CountingContract(Addr);

impl CountingContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: u64,
        label: &str,
        sender: &Addr,
        minimal_donation: Coin,
    ) -> StdResult<CountingContract> {
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg { minimal_donation },
            &[],
            label,
            None,
        )
        .map_err(|err| err.downcast().unwrap())
        .map(CountingContract)
    }

    #[track_caller]
    pub fn withdraw(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Withdraw {}, funds)
            .map_err(|err| err.downcast::<ContractError>().unwrap())?;
        Ok(())
    }

    #[track_caller]
    pub fn donate(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Donate {}, funds)
            .map_err(|err| err.downcast::<ContractError>().unwrap())?;
        Ok(())
    }

    pub fn query_value(&self, app: &App) -> StdResult<ValueResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Value {})
    }
}
