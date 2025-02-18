use crate::id::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    /// 集約のバージョン
    version: u128,
}

impl Aggregate {
    #[must_use]
    pub fn new(id: Id<Aggregate>, version: u128) -> Self {
        Self { id, version }
    }

    /// 集約のID
    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    /// 集約のバージョン
    #[must_use]
    pub fn version(&self) -> u128 {
        self.version
    }
}
