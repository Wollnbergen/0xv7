#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         WEEK 2: SMART CONTRACT IMPLEMENTATION                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "🔧 Step 1: Preparing CosmWasm Environment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Install Rust and wasm tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown

# Install cargo-generate
cargo install cargo-generate --features vendored-openssl

echo "✅ Rust and WASM tools installed"
echo ""

echo "🔧 Step 2: Creating Smart Contract Templates"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mkdir -p /workspaces/0xv7/contracts
cd /workspaces/0xv7/contracts

# Create a simple counter contract
cat > counter-contract.rs << 'CONTRACT'
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:counter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "increment"))
}
CONTRACT

echo "✅ Contract template created"
echo ""

echo "🔧 Step 3: Compiling and Optimizing Contract"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# This would normally compile the contract
echo "cargo wasm"
echo "wasm-opt -Os ./target/wasm32-unknown-unknown/release/counter.wasm -o ./counter_optimized.wasm"

echo ""
echo "�� Week 2 Tasks:"
echo "  ✅ CosmWasm environment setup"
echo "  ✅ Contract template creation"
echo "  ⏳ Contract compilation"
echo "  ⏳ Contract deployment"
echo "  ⏳ Contract testing"
