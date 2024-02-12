use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use lib::Error;
use ulid::Ulid;

pub mod aggregate;
pub mod command;
pub mod error;
pub mod event;
pub mod processor;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id<T> {
    value: Ulid,
    _maker: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn generate() -> Self {
        Self {
            value: Ulid::new(),
            _maker: PhantomData,
        }
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::generate()
    }
}

impl<T> FromStr for Id<T> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            value: Ulid::from_str(s)?,
            _maker: PhantomData,
        })
    }
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.to_string())
    }
}
