use std::marker::PhantomData;

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
