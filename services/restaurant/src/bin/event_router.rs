use adapter::EventModel;
use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};

async fn function_handler(event: LambdaEvent<Event>) -> Result<(), Error> {
    let changes: Vec<_> = event
        .payload
        .records
        .into_iter()
        .map(|x| x.change)
        .collect();
    for change in changes {
        let model: EventModel = serde_dynamo::from_item(change.new_image)?;
        tracing::info!("{model:?}");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
