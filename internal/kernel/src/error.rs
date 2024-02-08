use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AggregateError {
    #[error("Cannot update Aggregate version")]
    VersionUpdateLimitReached,
}
