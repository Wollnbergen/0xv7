use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Address(pub String);

impl Address {
    /// Creates a new Address with validation.
    /// Valid addresses must start with "sultan1" and be 45 characters (sultan1 + 38 alphanumeric).
    pub fn new(addr: &str) -> Result<Self, AddressError> {
        Self::validate(addr)?;
        Ok(Address(addr.to_string()))
    }

    /// Creates an Address without validation (for internal/test use).
    pub fn new_unchecked(addr: &str) -> Self {
        Address(addr.to_string())
    }

    /// Validates an address string.
    pub fn validate(addr: &str) -> Result<(), AddressError> {
        if !addr.starts_with("sultan1") {
            return Err(AddressError::InvalidPrefix);
        }
        if addr.len() != 45 {
            return Err(AddressError::InvalidLength(addr.len()));
        }
        // Check remaining characters are alphanumeric (after "sultan1")
        if !addr[7..].chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(AddressError::InvalidCharacters);
        }
        Ok(())
    }

    /// Returns the inner address string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Address {
    type Error = AddressError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(&s)
    }
}

impl TryFrom<&str> for Address {
    type Error = AddressError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    InvalidPrefix,
    InvalidLength(usize),
    InvalidCharacters,
}

impl std::fmt::Display for AddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressError::InvalidPrefix => write!(f, "address must start with 'sultan1'"),
            AddressError::InvalidLength(len) => write!(f, "address must be 45 characters, got {}", len),
            AddressError::InvalidCharacters => write!(f, "address contains invalid characters"),
        }
    }
}

impl std::error::Error for AddressError {}

// Re-export Transaction from blockchain module for backwards compatibility
pub use crate::blockchain::Transaction;
