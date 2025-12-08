#!/bin/bash

# Remove any duplicate create_wallet methods first
sed -i '/pub fn create_wallet.*{/,/^    }/d' node/src/database.rs

# Add proper imports at the top if missing
if ! grep -q "use anyhow::{anyhow, Result};" node/src/database.rs; then
    sed -i '1i\use anyhow::{anyhow, Result};' node/src/database.rs
fi

# Add the create_wallet method in impl Database block
# Find the impl Database block and add method there
cat >> temp_create_wallet.txt << 'METHOD'
    
    pub fn create_wallet(&mut self, address: &str) -> Result<()> {
        let full_address = if address.starts_with("sultan1") {
            address.to_string()
        } else {
            format!("sultan1{}", address)
        };
        
        if self.wallets.contains_key(&full_address) {
            return Ok(()); // Already exists, return success
        }
        
        let wallet = Wallet {
            address: full_address.clone(),
            balance: 0,
            nonce: 0,
            created_at: chrono::Utc::now().timestamp(),
            is_validator: false,
        };
        
        self.wallets.insert(full_address, wallet);
        Ok(())
    }
METHOD

# Insert the method before the closing brace of impl Database
sed -i '/^impl Database {/,/^}/ {
    /^}/ i\
'"$(cat temp_create_wallet.txt)"'
}' node/src/database.rs

rm temp_create_wallet.txt
echo "✅ Database.rs fixed"
