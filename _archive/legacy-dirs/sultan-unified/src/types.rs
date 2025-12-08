// Removed unused re-exports of Block and Transaction to satisfy clippy

#[derive(Debug)]
pub struct Address(pub String);

impl Address {
    pub fn new(addr: &str) -> Self {
        Address(addr.to_string())
    }
}
