#!/bin/bash

# Backup database.rs
cp node/src/database.rs node/src/database.rs.backup_day56

# Check if Transfer type exists, if not we need to define it inline
if ! grep -q "pub transfers:" node/src/database.rs; then
    # Add the transfers HashMap to Database struct
    sed -i '/pub stakes: HashMap<String, Stake>,/a\    pub transfers: HashMap<String, crate::token_transfer::Transfer>,' node/src/database.rs
    
    # Initialize transfers in new()
    sed -i '/stakes: HashMap::new(),/a\            transfers: HashMap::new(),' node/src/database.rs
fi

echo "✅ Database.rs updated"
