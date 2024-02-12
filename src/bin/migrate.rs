use lib::{database_url, Result};
use sqlx::{Connection, MySqlConnection};

#[tokio::main]
async fn main() -> Result<()> {
    let mut pool = MySqlConnection::connect(&database_url()).await?;
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
