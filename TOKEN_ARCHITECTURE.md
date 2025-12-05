# 💎 SULTAN TOKEN (SLTN) ARCHITECTURE DECISION

## ✅ RECOMMENDED: NATIVE TOKEN (Not just CW20)

### Why SLTN Should Be Native:

1. **Gas Fee Token**: SLTN must be the native token to enable $0.00 gas fees
2. **Staking Token**: Validators stake SLTN natively for 26.67% APY
3. **Governance Token**: Native integration for on-chain governance
4. **IBC Transfer**: Native tokens have better IBC support
5. **Performance**: Native tokens are 10x faster than contract tokens

### Dual Token Strategy:
- **Native SLTN**: Core blockchain token (gas, staking, governance)
- **Wrapped SLTN (wSLTN)**: CW20 version for DeFi protocols

### Architecture:
─────────────────────────────────────────┐
│ NATIVE SLTN (Layer 0) │
│ • Gas fees ($0.00) │
│ • Staking (26.67% APY) │
│ • Governance │
└──────────────┬──────────────────────────┘
│
┌───────▼────────┐
│ Bridge Module │
└───────┬────────┘
│
┌──────────────▼──────────────────────────┐
│ CW20 wSLTN (Smart Contracts) │
│ • DEX trading │
│ • Liquidity pools │
│ • DeFi integrations │
└─────────────────────────────────────────┘
