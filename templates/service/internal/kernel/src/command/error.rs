use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommandKernelError {}
