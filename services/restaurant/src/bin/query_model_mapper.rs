use adapter::model::event::Payload;
use adapter::EventModel;
use aws_lambda_events::event::sqs::SqsEvent;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use sqlx::{Connection, MySqlConnection};

async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    const QUERY_INSERT_RESTAURANT: &str =
        "INSERT INTO restaurant (aggregate_id, restaurant_name) VALUES (?, ?)";
    const QUERY_INSERT_ITEM: &str =
        "INSERT INTO restaurant_item (aggregate_id, item_id, item_name, price) VALUES (?, ?, ?, ?)";
    const QUERY_DELETE_ITEM: &str = "DELETE FROM restaurant_item WHERE item_id = ?";

    let Ok(mysql_url) = std::env::var("MYSQL_URL") else {
        return Err("MYSQL_URL must be set".into());
    };
    let mut pool = MySqlConnection::connect(&mysql_url).await?;

    let mut models = Vec::new();
    let mut errs: Vec<Error> = Vec::new();
    for record in event.payload.records {
        if record.body.is_none() {
            errs.push("event body is None".into());
            continue;
        }
        match serde_json::from_str::<EventModel>(&record.body.unwrap()) {
            Ok(v) => models.push(v),
            Err(e) => errs.push(e.into()),
        }
    }

    let mut tx = pool.begin().await?;
    for model in models {
        match model.payload() {
            Payload::AggregateCreatedV1(restaurant) => {
                sqlx::query(QUERY_INSERT_RESTAURANT)
                    .bind(model.aggregate_id().as_bytes())
                    .bind(restaurant.name())
                    .execute(&mut *tx)
                    .await?;
            }
            Payload::ItemsAddedV1(items) => {
                for item in items {
                    sqlx::query(QUERY_INSERT_ITEM)
                        .bind(model.aggregate_id().as_bytes())
                        .bind(item.id().as_bytes())
                        .bind(item.name())
                        .bind(item.price().value())
                        .execute(&mut *tx)
                        .await?;
                }
            }
            Payload::ItemsRemovedV1(item_ids) => {
                for id in item_ids {
                    sqlx::query(QUERY_DELETE_ITEM)
                        .bind(id)
                        .execute(&mut *tx)
                        .await?;
                }
            }
        }
    }
    tx.commit().await?;

    if !errs.is_empty() {
        for err in &errs {
            tracing::error!(err);
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
