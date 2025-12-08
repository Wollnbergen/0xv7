#!/bin/bash

# Add Debug derive to Database struct
sed -i 's/pub struct Database {/#[derive(Debug)]\npub struct Database {/' node/src/database.rs

# Make sure transfers field exists and is properly typed
if ! grep -q "pub transfers:" node/src/database.rs; then
    # First, let's define a simple Transfer type in database.rs to avoid circular dependencies
    sed -i '1i\use std::collections::HashMap;' node/src/database.rs
    
    # Add a simple Transfer definition in database.rs
    cat >> temp_transfer_def.txt << 'TRANSFER'

// Simplified Transfer type for database
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transfer {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub timestamp: i64,
    pub memo: Option<String>,
}
TRANSFER
    
    # Insert before Database struct
    sed -i '/pub struct Database/r temp_transfer_def.txt' node/src/database.rs
    
    # Add transfers field to Database
    sed -i '/pub stakes: HashMap<String, Stake>,/a\    pub transfers: HashMap<String, Transfer>,' node/src/database.rs
fi

# Initialize transfers in new() if not done
if ! grep -q "transfers: HashMap::new()" node/src/database.rs; then
    sed -i '/stakes: HashMap::new(),/a\            transfers: HashMap::new(),' node/src/database.rs
fi

echo "✅ Database.rs structure fixed"
