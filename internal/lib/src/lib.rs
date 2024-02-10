pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = core::result::Result<T, Error>;
