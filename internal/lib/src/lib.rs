pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub fn database_url() -> String {
    format!(
        "mysql://{}:{}@127.0.0.1:{}/{}",
        option_env!("MYSQL_USERNAME").unwrap_or("root"),
        option_env!("MYSQL_PASSWORD").unwrap_or("root"),
        option_env!("MYSQL_PORT").unwrap_or("3306"),
        option_env!("MYSQL_DATABASE").unwrap_or("widget")
    )
}

pub fn application_environment() -> String {
    option_env!("APPLICATION_ENVIRONMENT")
        .unwrap_or("development")
        .to_string()
}

pub fn opentelemetry_endpoint() -> String {
    format!(
        "http://{}:{}",
        option_env!("OPENTELEMETRY_HOST").unwrap_or("127.0.0.1"),
        option_env!("OPENTELEMETRY_PORT").unwrap_or("4317")
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg(feature = "test")]
pub enum DateTime {
    /// 2023-01-01T00:00:00Z
    DT2023_01_01_00_00_00_00,
    /// 2023-01-01T00:00:01Z
    DT2023_01_01_00_00_00_01,
    /// 2023-01-01T00:00:02Z
    DT2023_01_01_00_00_00_02,
    /// 2023-01-01T00:01:00Z
    DT2023_01_01_00_00_01_00,
    /// 2023-01-01T01:00:00Z
    DT2023_01_01_00_01_00_00,
    /// 2023-01-01T01:00:00Z
    DT2023_01_01_01_00_00_00,
    /// 2023-01-02T00:00:00Z
    DT2023_01_02_00_00_00_00,
    /// 2023-02-01T00:00:00Z
    DT2023_02_01_00_00_00_00,
    /// 2024-01-01T00:00:00Z
    DT2024_01_01_00_00_00_00,
}

#[cfg(feature = "test")]
impl DateTime {
    pub fn id(self) -> String {
        match self {
            DateTime::DT2023_01_01_00_00_00_00 => "01GNNA1J00PQ9J874NBWERBM3Z",
            DateTime::DT2023_01_01_00_00_00_01 => "01GNNA1J015CFH0CA590B4K9K6",
            DateTime::DT2023_01_01_00_00_00_02 => "01GNNA1J02N9H1YCMRA2R9Q562",
            DateTime::DT2023_01_01_00_00_01_00 => "01GNNA1JZ86A6F1G8HV7NYHDCN",
            DateTime::DT2023_01_01_00_01_00_00 => "01GNNA3CK0B63HH8HBYQVRJ5Y8",
            DateTime::DT2023_01_01_01_00_00_00 => "01GNNDFDM0WV3PR6RM8TEA7MZ5",
            DateTime::DT2023_01_02_00_00_00_00 => "01GNQWE9003DQHKPAAHCDCVTJZ",
            DateTime::DT2023_02_01_00_00_00_00 => "01GR57SPM0XBGEG4A13ZBW02G2",
            DateTime::DT2024_01_01_00_00_00_00 => "01HK153X00D14NM09FKYEJ7MPY",
        }
        .to_string()
    }
}

#[cfg(feature = "test")]
pub async fn test_client() -> aws_sdk_dynamodb::Client {
    let config = aws_config::defaults(aws_config::BehaviorVersion::v2024_03_28())
        .endpoint_url("http://localhost:8000")
        .test_credentials()
        .load()
        .await;
    aws_sdk_dynamodb::Client::new(&config)
}
