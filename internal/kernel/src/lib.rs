use std::marker::PhantomData;

use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id<T> {
    value: Ulid,
    _maker: PhantomData<T>,
}
