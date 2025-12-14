use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, Addr,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_supply: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Transfer { recipient: String, amount: Uint128 },
    Burn { amount: Uint128 },
    Mint { recipient: String, amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Balance { address: String },
    TokenInfo {},
}

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("name", msg.name)
        .add_attribute("symbol", msg.symbol))
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            Ok(Response::new()
                .add_attribute("method", "transfer")
                .add_attribute("recipient", recipient)
                .add_attribute("amount", amount.to_string()))
        },
        ExecuteMsg::Burn { amount } => {
            Ok(Response::new()
                .add_attribute("method", "burn")
                .add_attribute("amount", amount.to_string()))
        },
        ExecuteMsg::Mint { recipient, amount } => {
            Ok(Response::new()
                .add_attribute("method", "mint")
                .add_attribute("recipient", recipient)
                .add_attribute("amount", amount.to_string()))
        }
    }
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => {
            let balance = Uint128::new(1000000);
            to_json_binary(&balance)
        },
        QueryMsg::TokenInfo {} => {
            to_json_binary(&"Sultan Token")
        }
    }
}
