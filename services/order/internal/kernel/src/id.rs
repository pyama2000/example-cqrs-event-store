use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use uuid::Uuid;

/// Generic UUID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<T> {
    value: Uuid,
    _maker: PhantomData<T>,
}

impl<T> Id<T> {
    /// Create new Id
    #[must_use]
    pub fn generate() -> Self {
        Self {
            value: Uuid::new_v4(),
            _maker: PhantomData,
        }
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Id::generate()
    }
}

impl<T> FromStr for Id<T> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use anyhow::Context as _;

        Ok(Self {
            value: Uuid::from_str(s).with_context(|| "parse UUID from string")?,
            _maker: PhantomData,
        })
    }
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
