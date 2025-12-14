use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw721_base::{ContractError, Extension, InstantiateMsg, ExecuteMsg, QueryMsg};

// Custom metadata for Sultan NFTs
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
pub struct SultanMetadata {
    pub power_level: u64,
    pub rarity: String,
    pub zero_gas: bool,
}

// Instantiate NFT contract
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let contract = cw721_base::Cw721Contract::<Extension, Extension, Extension, Extension>::default();
    contract.instantiate(deps, env, info, msg)
}

// Execute NFT operations (ZERO GAS!)
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Extension, Extension>,
) -> Result<Response, ContractError> {
    let contract = cw721_base::Cw721Contract::<Extension, Extension, Extension, Extension>::default();
    contract.execute(deps, env, info, msg)
}

// Query NFT data
#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<Extension>) -> StdResult<Binary> {
    let contract = cw721_base::Cw721Contract::<Extension, Extension, Extension, Extension>::default();
    contract.query(deps, env, msg)
}
