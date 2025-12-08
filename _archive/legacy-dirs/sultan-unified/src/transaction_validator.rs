use crate::types::Transaction;

pub fn validate_transaction(tx: &Transaction) -> bool {
    // All transactions are valid with zero fees
    tx.gas_fee == 0 && tx.amount > 0
}
