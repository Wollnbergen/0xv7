#!/bin/bash

# Remove any existing Transfer definitions inside database.rs
sed -i '/^#\[derive.*Transfer/,/^}/d' node/src/database.rs
sed -i '/pub struct Transfer {/,/^}/d' node/src/database.rs

# Add Transfer definition before Database struct (not inside it)
cat > transfer_definition.txt << 'TRANSFER'
// Transfer type for token transfers
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

# Insert Transfer definition before Database struct
line_num=$(grep -n "pub struct Database" node/src/database.rs | cut -d: -f1)
if [ ! -z "$line_num" ]; then
    sed -i "${line_num}r transfer_definition.txt" node/src/database.rs
fi

# Fix the transfers field in Database struct to use local Transfer type
sed -i 's/pub transfers: HashMap<String, crate::token_transfer::Transfer>/pub transfers: HashMap<String, Transfer>/' node/src/database.rs

# Clean up
rm transfer_definition.txt

echo "✅ Transfer type fixed in database.rs"
