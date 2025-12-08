# Sultan Chain Token Architecture

## Layer 0: Native Token
SLTN (Native)
├── Gas payments ($0.00 fees)
├── Validator staking (13.33% APY)
├── Governance voting
└── IBC transfers

## Layer 1: Smart Contract Tokens (CW20)
Wrapped Tokens
├── wSLTN (1:1 with native SLTN)
└── Bridge contract manages wrapping/unwrapping

Ecosystem Tokens
├── USDS (USD stablecoin)
├── GOLD (commodity token)
└── SLP (LP tokens)

## Layer 2: DeFi Protocols
AMM DEX
├── SLTN/USDS pool
├── SLTN/GOLD pool
├── USDS/GOLD pool
└── All with $0.00 gas fees


