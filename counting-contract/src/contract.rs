use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};
use crate::msg::InstantiateMsg;
use crate::state::{COUNTER, MINIMAL_DONATION, OWNER};

pub fn instantiate(deps: DepsMut, msg: InstantiateMsg, info: MessageInfo) -> StdResult<Response> {
    COUNTER.save(deps.storage, &0)?;
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

        Ok(ValueResp { value})
    }
}

pub mod exec {
    use cosmwasm_std::{BankMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
    use crate::error::ContractError;
    use crate::state::{COUNTER, MINIMAL_DONATION, OWNER};

    pub fn donate(
        deps: DepsMut,
        info: MessageInfo
    ) -> StdResult<Response> {
        let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;

        let mut value = COUNTER.load(deps.storage)?;

        if info.funds.iter().any(|coin| {
           coin.denom == minimal_donation.denom && coin.amount >= minimal_donation.amount
        }) {
            value += 1;
            COUNTER.save(deps.storage, &value)?;
        }

        let resp = Response::new()
            .add_attribute("action", "poke")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", value.to_string());
        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {owner: owner.into() });
        }

        let funds = deps.querier.query_all_balances(env.contract.address)?;
        let bank_msg = BankMsg::Send { to_address: owner.to_string(), amount: funds};
        let resp = Response::new().add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        return Ok(resp)
    }
}