use lambda_runtime::tracing;

#[tracing::instrument(err, ret)]
async fn handler(
    event: lambda_runtime::LambdaEvent<aws_lambda_events::event::dynamodb::Event>,
) -> Result<(), lambda_runtime::Error> {
    use adapter::command::model::{EventPayload, EventStoreModel};
    use anyhow::Context as _;
    use proto::cart::v1::cart_service_client::CartServiceClient;
    use proto::order::v1::order_service_client::OrderServiceClient;

    let models: Vec<EventStoreModel> = event
        .payload
        .records
        .into_iter()
        .map(|record| serde_dynamo::from_item(record.change.new_image))
        .collect::<Result<_, _>>()?;
    for model in &models {
        let metadata = model.metadata();
        let cx = observability::aws_dynamodb::extract(metadata);
        let span = tracing::Span::current();
        observability::aws_lambda::add_link(cx, &span);
    }
    let order_placed_models: Vec<_> = models
        .into_iter()
        .filter(|model| model.payload() == &EventPayload::OrderPlacedV1)
        .collect();
    let cart_service_endpoint =
        std::env::var("CART_SERVICE_ENDPOINT").context("CART_SERVICE_ENDPOINT must be set")?;
    let mut cart_service = CartServiceClient::connect(format!("http://{cart_service_endpoint}"))
        .await
        .context("connect cart service")?;
    let order_service_endpoint =
        std::env::var("ORDER_SERVICE_ENDPOINT").context("ORDER_SERVICE_ENDPOINT must be set")?;
    let mut order_service = OrderServiceClient::connect(format!("http://{order_service_endpoint}"))
        .await
        .context("connect order service")?;
    for model in order_placed_models {
        let cart_id = model.aggregate_id().to_string();
        let message = proto::cart::v1::GetRequest {
            id: cart_id.clone(),
        };
        let mut request = tonic::Request::new(message.clone());
        request.set_timeout(std::time::Duration::from_millis(500));
        observability::grpc_client::inject(
            &observability::grpc_client::context(),
            request.metadata_mut(),
        );
        let response = cart_service
            .get(request)
            .await
            .with_context(|| format!("Call cart.v1.CartService/Get: {message:?}"))?;
        let items: Vec<_> = response
            .into_inner()
            .items
            .into_iter()
            .map(|item| {
                let proto::cart::v1::get_response::Item {
                    tenant_id,
                    item_id,
                    quantity,
                } = item;
                proto::order::v1::Item {
                    tenant_id,
                    item_id,
                    quantity,
                }
            })
            .collect();
        let message = proto::order::v1::CreateRequest { cart_id, items };
        let mut request = tonic::Request::new(message.clone());
        request.set_timeout(std::time::Duration::from_millis(500));
        observability::grpc_client::inject(
            &observability::grpc_client::context(),
            request.metadata_mut(),
        );
        order_service
            .create(request)
            .await
            .with_context(|| format!("Call order.v1.OrderService/Create: {message:?}"))?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let force_flush = observability::provider::init_providers_with_flush(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )?;
    observability::aws_lambda::run(
        lambda_runtime::service_fn(handler),
        force_flush,
        env!("CARGO_PKG_VERSION").to_string(),
    )
    .await
}
