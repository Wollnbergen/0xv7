pub use crate::blockchain::Transaction;
pub use crate::blockchain::Block;

#[derive(Debug)]
pub struct Address(pub String);

impl Address {
    pub fn new(addr: &str) -> Self {
        Address(addr.to_string())
    }
}
