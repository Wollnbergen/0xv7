#!/bin/bash

# Remove any existing create_wallet methods
sed -i '/pub fn create_wallet/,/^    }/d' node/src/database.rs

# Find the impl Database block and add the method properly
cat > create_wallet_method.txt << 'METHOD'

    pub fn create_wallet(&mut self, address: &str) -> anyhow::Result<()> {
        let full_address = if address.starts_with("sultan1") {
            address.to_string()
        } else {
            format!("sultan1{}", address)
        };
        
        if self.wallets.contains_key(&full_address) {
            return Ok(()); // Already exists
        }
        
        use crate::types::Wallet;
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

# Find where to insert (before the last } of impl Database)
impl_start=$(grep -n "^impl Database" node/src/database.rs | cut -d: -f1)
if [ ! -z "$impl_start" ]; then
    # Find the matching closing brace
    impl_end=$(awk -v start="$impl_start" 'NR >= start && /^}/ {print NR; exit}' node/src/database.rs)
    if [ ! -z "$impl_end" ]; then
        sed -i "${impl_end}i\\$(cat create_wallet_method.txt)" node/src/database.rs
    fi
fi

rm create_wallet_method.txt
echo "✅ create_wallet method added"
