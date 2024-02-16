use lib::Error;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};

pub type ConnectionPool = Pool<MySql>;

pub async fn connect(url: &str) -> Result<ConnectionPool, Error> {
    Ok(MySqlPoolOptions::new()
        .max_connections(20)
        .connect(url)
        .await?)
}
