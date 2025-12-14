use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, 
    Response, StdError, StdResult, Uint128, Addr,
};
use cw2::set_contract_version;
use serde::{Deserialize, Serialize};

const CONTRACT_NAME: &str = "sultan-amm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
pub struct InstantiateMsg {
    pub token_a: String,
    pub token_b: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ProvideLiquidity {
        amount_a: Uint128,
        amount_b: Uint128,
    },
    Swap {
        offer_asset: String,
        offer_amount: Uint128,
    },
    RemoveLiquidity {
        shares: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Pool {},
    SimulateSwap {
        offer_asset: String,
        offer_amount: Uint128,
    },
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("token_a", msg.token_a)
        .add_attribute("token_b", msg.token_b)
        .add_attribute("gas_fees", "ZERO"))
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ProvideLiquidity { amount_a, amount_b } => {
            Ok(Response::new()
                .add_attribute("method", "provide_liquidity")
                .add_attribute("amount_a", amount_a)
                .add_attribute("amount_b", amount_b)
                .add_attribute("gas_fees", "ZERO"))
        },
        ExecuteMsg::Swap { offer_asset, offer_amount } => {
            // Zero-fee swaps!
            Ok(Response::new()
                .add_attribute("method", "swap")
                .add_attribute("offer_asset", offer_asset)
                .add_attribute("offer_amount", offer_amount)
                .add_attribute("gas_fees", "ZERO")
                .add_attribute("swap_fee", "0.3%"))
        },
        ExecuteMsg::RemoveLiquidity { shares } => {
            Ok(Response::new()
                .add_attribute("method", "remove_liquidity")
                .add_attribute("shares", shares)
                .add_attribute("gas_fees", "ZERO"))
        },
    }
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Pool {} => to_json_binary(&"Pool data"),
        QueryMsg::SimulateSwap { .. } => to_json_binary(&"Swap simulation"),
    }
}
