#!/bin/bash

echo "⛓️ Creating Sultan Chain Genesis Block..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create genesis configuration
cat > /workspaces/0xv7/sultan-mainnet/config/genesis.json << 'JSON'
{
  "genesis_time": "2025-11-02T08:00:00.000000Z",
  "chain_id": "sultan-mainnet-1",
  "initial_height": "1",
  "consensus_params": {
    "block": {
      "max_bytes": "22020096",
      "max_gas": "0",
      "time_iota_ms": "5000"
    }
  },
  "validators": [
    {
      "address": "sultanvaloper1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq",
      "pub_key": {
        "type": "tendermint/PubKeyEd25519",
        "value": "oWg2ISpLF423gSd/v3F9pVsJEeEVxSJVPCxjlU1+UkE="
      },
      "power": "100000000",
      "name": "Sultan Foundation Validator",
      "is_mobile": false
    }
  ],
  "app_state": {
    "auth": {
      "params": {
        "max_memo_characters": "256",
        "tx_sig_limit": "7",
        "tx_size_cost_per_byte": "0",
        "sig_verify_cost_ed25519": "0",
        "sig_verify_cost_secp256k1": "0"
      }
    },
    "bank": {
      "params": {
        "send_enabled": true
      }
    },
    "distribution": {
      "params": {
        "base_proposer_reward": "0.01",
        "bonus_proposer_reward": "0.04",
        "withdraw_addr_enabled": true
      }
    },
    "mint": {
      "minter": {
        "inflation": "0.08",
        "annual_provisions": "0.000000000000000000"
      },
      "params": {
        "mint_denom": "usltn",
        "inflation_rate_change": "0.00",
        "inflation_max": "0.08",
        "inflation_min": "0.08",
        "goal_bonded": "0.30",
        "blocks_per_year": "6311520"
      }
    },
    "staking": {
      "params": {
        "unbonding_time": "1814400s",
        "max_validators": 100,
        "max_entries": 7,
        "historical_entries": 10000,
        "bond_denom": "usltn",
        "min_commission_rate": "0.000000000000000000",
        "validator_bonus_mobile": "0.40"
      }
    },
    "slashing": {
      "params": {
        "signed_blocks_window": "100",
        "min_signed_per_window": "0.500000000000000000",
        "downtime_jail_duration": "600s",
        "slash_fraction_double_sign": "0.050000000000000000",
        "slash_fraction_downtime": "0.010000000000000000"
      }
    },
    "gov": {
      "voting_params": {
        "voting_period": "172800s"
      },
      "deposit_params": {
        "min_deposit": [
          {
            "denom": "usltn",
            "amount": "10000000"
          }
        ],
        "max_deposit_period": "172800s"
      }
    },
    "crisis": {
      "constant_fee": {
        "denom": "usltn",
        "amount": "0"
      }
    },
    "feegrant": {
      "allowances": []
    },
    "params": null,
    "upgrade": {},
    "vesting": {},
    "wasm": {
      "params": {
        "code_upload_access": {
          "permission": "Everybody"
        },
        "instantiate_default_permission": "Everybody"
      }
    }
  }
}
JSON

echo "✅ Genesis block created with:"
echo "  • Chain ID: sultan-mainnet-1"
echo "  • Zero gas fees configured"
echo "  • 4% inflation rate"
echo "  • 30% target staking ratio"
echo "  • 40% mobile validator bonus"
echo "  • 5-second block times"

