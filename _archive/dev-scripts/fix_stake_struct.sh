#!/bin/bash
# Add last_claim_timestamp to Stake struct
sed -i '/pub timestamp: i64,/a\    pub last_claim_timestamp: Option<i64>,' node/src/types.rs
