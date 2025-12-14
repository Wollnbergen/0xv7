use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw20_base::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw20_base::{ContractError, state::{MinterData, TokenInfo, BALANCES, TOKEN_INFO}};

// Entry point for instantiation
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Override with Sultan Token details
    msg.name = "Sultan Token".to_string();
    msg.symbol = "SLTN".to_string();
    msg.decimals = 6;
    
    // Use cw20-base instantiate
    cw20_base::contract::instantiate(deps, env, info, msg)
}

// Entry point for execution (ZERO GAS FEES!)
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // All transactions are FREE!
    cw20_base::contract::execute(deps, env, info, msg)
}

// Entry point for queries
#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw20_base::contract::query(deps, env, msg)
}
