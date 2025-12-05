use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    StdError, Addr,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Increment {},
    Reset { count: i32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetCount {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CountResponse {
    pub count: i32,
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    deps.storage.set(b"state", &to_binary(&state)?);
    
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Increment {} => {
            let data = deps.storage.get(b"state").ok_or_else(|| StdError::not_found("State"))?;
            let mut state: State = cosmwasm_std::from_binary(&data)?;
            state.count += 1;
            deps.storage.set(b"state", &to_binary(&state)?);
            
            Ok(Response::new()
                .add_attribute("method", "increment")
                .add_attribute("count", state.count.to_string()))
        }
        ExecuteMsg::Reset { count } => {
            let data = deps.storage.get(b"state").ok_or_else(|| StdError::not_found("State"))?;
            let mut state: State = cosmwasm_std::from_binary(&data)?;
            state.count = count;
            deps.storage.set(b"state", &to_binary(&state)?);
            
            Ok(Response::new()
                .add_attribute("method", "reset")
                .add_attribute("count", count.to_string()))
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => {
            let data = deps.storage.get(b"state").ok_or_else(|| StdError::not_found("State"))?;
            let state: State = cosmwasm_std::from_binary(&data)?;
            to_binary(&CountResponse { count: state.count })
        }
    }
}
