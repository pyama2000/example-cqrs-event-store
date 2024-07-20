use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};

async fn function_handler(event: LambdaEvent<Event>) -> Result<(), Error> {
    tracing::info!("{event:?}");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
