use lib::Result;
use sqlx::{Connection, MySqlConnection};

#[tokio::main]
async fn main() -> Result<()> {
    let mut pool = MySqlConnection::connect("mysql://root:root@127.0.0.1:3306/widget").await?;
    sqlx::query(include_str!(
        "../../migrations/20240210132634_create_aggregate.sql"
    ))
    .execute(&mut pool)
    .await?;
    sqlx::query(include_str!(
        "../../migrations/20240210132646_create_event.sql"
    ))
    .execute(&mut pool)
    .await?;
    Ok(())
}
