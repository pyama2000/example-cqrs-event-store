use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueryKernelError {}
