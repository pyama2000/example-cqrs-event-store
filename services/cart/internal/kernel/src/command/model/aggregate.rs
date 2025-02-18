#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Aggregate {
    id: (),
    /// 集約のバージョン
    version: u128,
}

impl Aggregate {
    #[must_use]
    pub fn new(id: (), version: u128) -> Self {
        Self { id, version }
    }

    /// 集約のID
    #[must_use]
    pub fn id(&self) -> &() {
        &self.id
    }

    /// 集約のバージョン
    #[must_use]
    pub fn version(&self) -> u128 {
        self.version
    }
}
