use adapter::EventModel;
use aws_config::BehaviorVersion;
use aws_lambda_events::event::dynamodb::Event;
use aws_sdk_sqs::types::SendMessageBatchRequestEntry;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};

async fn function_handler(event: LambdaEvent<Event>) -> Result<(), Error> {
    let Ok(query_model_mapper_queue_url) = std::env::var("QUERY_MODEL_MAPPER_QUEUE_URL") else {
        return Err("QUERY_MODEL_MAPPER_QUEUE_URL must be set".into());
    };
    let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
    let sqs = aws_sdk_sqs::Client::new(&config);

    let (oks, errs): (Vec<_>, Vec<_>) = event
        .payload
        .records
        .into_iter()
        .map(|x| serde_dynamo::from_item::<serde_dynamo::Item, EventModel>(x.change.new_image))
        .partition(Result::is_ok);

    let models: Vec<_> = oks.into_iter().flatten().collect();
    let mut entries = Vec::new();
    for model in models {
        let entry = SendMessageBatchRequestEntry::builder()
            .id(model.id())
            .message_group_id(model.aggregate_id())
            .message_body(serde_json::to_string(&model)?)
            .build()?;
        entries.push(entry);
    }
    sqs.send_message_batch()
        .queue_url(query_model_mapper_queue_url)
        .set_entries(Some(entries))
        .send()
        .await?;

    if !errs.is_empty() {
        for err in &errs {
            if let Err(e) = err {
                tracing::error!("{e:?}");
            }
        }
        return Err("errors occured!".into());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
