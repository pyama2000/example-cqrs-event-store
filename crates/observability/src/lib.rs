#[cfg(feature = "aws-lambda")]
pub mod aws_lambda;
#[cfg(feature = "provider")]
pub mod provider;
#[cfg(feature = "server")]
pub mod server;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
