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
