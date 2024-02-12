use adapter::persistence::connect;
use adapter::repository::WidgetRepository;
use app::WidgetServiceImpl;
use driver::Server;
use lib::{database_url, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let pool = connect(&database_url()).await?;
    let repository = WidgetRepository::new(pool);
    let service = WidgetServiceImpl::new(repository);
    let server = Server::new("0.0.0.0:8080", service.into());
    server.run().await
}
