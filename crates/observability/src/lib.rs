#[cfg(feature = "aws-dynamodb")]
pub mod aws_dynamodb;
#[cfg(feature = "aws-lambda")]
pub mod aws_lambda;
#[cfg(feature = "grpc-client")]
pub mod grpc_client;
#[cfg(feature = "provider")]
pub mod provider;
#[cfg(feature = "server")]
pub mod server;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
