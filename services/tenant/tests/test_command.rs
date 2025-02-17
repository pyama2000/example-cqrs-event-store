use adapter::QueryRepository;
use adapter::{dynamodb, CommandRepository};
use app::CommandUseCase;
use app::QueryUseCase;
use aws_config::BehaviorVersion;
use backon::{ExponentialBuilder, Retryable};
use driver::server::Server;
use driver::server::Service;
use proto::tenant::v1::add_items_request::Item;
use proto::tenant::v1::tenant_service_client::TenantServiceClient;
use proto::tenant::v1::{
    AddItemsRequest, CreateRequest, ListItemsRequest, ListTenantsRequest, RemoveItemsRequest,
};
use rand::Rng;
use testcontainers::ContainerAsync;
use testcontainers_modules::dynamodb_local::DynamoDb;
use tonic::transport::Channel;
use tonic::Code;
use tonic_health::pb::{health_client::HealthClient, HealthCheckRequest};
use tonic_health::ServingStatus;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::test]
async fn test_with_container_command_ok() -> Result<(), Error> {
    let Context {
        container: _container,
        mut client,
    } = Context::with_container().await?;
    let res = client
        .create(CreateRequest {
            name: "テストテナント".to_string(),
        })
        .await?;
    let tenant_id = res.into_inner().id;
    let res = client.list_tenants(ListTenantsRequest {}).await?;
    assert_eq!(res.into_inner().tenants.len(), 1);
    let res = client
        .add_items(AddItemsRequest {
            tenant_id: tenant_id.clone(),
            items: vec![
                Item {
                    name: "テスト商品1".to_string(),
                    price: 1000,
                },
                Item {
                    name: "テスト商品2".to_string(),
                    price: 2000,
                },
                Item {
                    name: "テスト商品3".to_string(),
                    price: 3000,
                },
            ],
        })
        .await?;
    let item_ids = res.into_inner().ids;
    let res = client
        .list_items(ListItemsRequest {
            tenant_id: tenant_id.clone(),
        })
        .await?;
    assert_eq!(
        item_ids,
        res.into_inner()
            .items
            .into_iter()
            .map(|x| x.id)
            .collect::<Vec<_>>()
    );
    client
        .remove_items(RemoveItemsRequest {
            tenant_id,
            item_ids: vec![item_ids.get(1).ok_or("item id not found")?.to_string()],
        })
        .await?;
    Ok(())
}

#[tokio::test]
async fn test_command_create_err() -> Result<(), Error> {
    struct TestCase {
        name: &'static str,
        request: CreateRequest,
        expected: Code,
    }

    let Context { mut client, .. } = Context::without_container().await?;

    let tests = [TestCase {
        name: "テナント名が空文字の場合はInvalidArgumentが返る",
        request: CreateRequest {
            name: String::new(),
        },
        expected: Code::InvalidArgument,
    }];
    for TestCase {
        name,
        request,
        expected,
    } in tests
    {
        let result = client.create(request).await;
        assert!(result.is_err(), "{name}: result must be error: {result:#?}");
        assert_eq!(result.err().unwrap().code(), expected, "{name}");
    }

    Ok(())
}

struct Context {
    container: Option<ContainerAsync<DynamoDb>>,
    client: TenantServiceClient<Channel>,
}

impl Context {
    #[allow(clippy::too_many_lines)]
    async fn with_container() -> Result<Self, Error> {
        use adapter::{AGGREGATE_TABLE_NAME, EVENT_SEQUENCE_TABLE_NAME, EVENT_STORE_TABLE_NAME};
        use aws_sdk_dynamodb::types::{
            AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ScalarAttributeType,
        };
        use testcontainers::runners::AsyncRunner;

        // NOTE: デフォルトのDockerコンテキスト以外を使っている場合にtestcontainersが正しく動作しないため、
        // 環境変数の `DOCKER_HOST` にホストを設定する必要がある
        // read mores: https://github.com/testcontainers/testcontainers-rs/issues/627
        option_env!("DOCKER_HOST").unwrap_or_else(|| panic!("DOCKER_HOST must be set (e.g. DOCKER_HOST=(docker context inspect | jq -r '.[0].Endpoints.docker.Host'))"));

        // Amazon DynamoDB localのコンテナを起動する
        let container = DynamoDb::default().start().await?;
        let endpoint = format!(
            "http://{}:{}",
            container.get_host().await?,
            container.get_host_port_ipv4(8000).await?
        );
        let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .endpoint_url(endpoint)
            .test_credentials()
            .load()
            .await;
        let dynamodb = dynamodb(&config);

        // Amazon DynamoDBのテーブルを作成する
        dynamodb
            .create_table()
            .table_name(EVENT_STORE_TABLE_NAME)
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("aggregate_id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("aggregate_id")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(ScalarAttributeType::N)
                    .build()?,
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(KeyType::Range)
                    .build()?,
            )
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await?;
        dynamodb
            .create_table()
            .table_name(EVENT_SEQUENCE_TABLE_NAME)
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("aggregate_id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("aggregate_id")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await?;
        dynamodb
            .create_table()
            .table_name(AGGREGATE_TABLE_NAME)
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await?;

        let server = Server::new(Service::new(
            CommandUseCase::new(CommandRepository::new(dynamodb.clone())),
            QueryUseCase::new(QueryRepository::new(dynamodb)),
        ));
        let port = {
            let mut rng = rand::thread_rng();
            rng.gen_range(50000..60000)
        };
        let addr = format!("[::1]:{port}").parse()?;
        // gRPCサーバーを起動する
        tokio::spawn(async move {
            server.run(addr).await?;
            Ok::<_, Error>(())
        });
        // gRPCサーバーが起動している状態か確認する
        let health_check = || async {
            let conn = tonic::transport::Endpoint::new(format!("http://{addr}"))?.connect_lazy();
            let mut client = HealthClient::new(conn);
            let res = client.check(HealthCheckRequest::default()).await?;
            if res.into_inner().status() != ServingStatus::Serving.into() {
                return Err("not served".into());
            }
            Ok::<_, Error>(())
        };
        health_check
            .retry(ExponentialBuilder::default())
            .sleep(tokio::time::sleep)
            .await?;
        let client = TenantServiceClient::connect(format!("http://{addr}")).await?;
        Ok(Self {
            container: Some(container),
            client,
        })
    }

    async fn without_container() -> Result<Self, Error> {
        let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .test_credentials()
            .load()
            .await;
        let dynamodb = dynamodb(&config);
        let server = Server::new(Service::new(
            CommandUseCase::new(CommandRepository::new(dynamodb.clone())),
            QueryUseCase::new(QueryRepository::new(dynamodb)),
        ));
        let port = {
            let mut rng = rand::thread_rng();
            rng.gen_range(50000..60000)
        };
        let addr = format!("[::1]:{port}").parse()?;
        // gRPCサーバーを起動する
        tokio::spawn(async move {
            server.run(addr).await?;
            Ok::<_, Error>(())
        });
        // gRPCサーバーが起動している状態か確認する
        let health_check = || async {
            let conn = tonic::transport::Endpoint::new(format!("http://{addr}"))?.connect_lazy();
            let mut client = HealthClient::new(conn);
            let res = client.check(HealthCheckRequest::default()).await?;
            if res.into_inner().status() != ServingStatus::Serving.into() {
                return Err("not served".into());
            }
            Ok::<_, Error>(())
        };
        health_check
            .retry(ExponentialBuilder::default())
            .sleep(tokio::time::sleep)
            .await?;
        let client = TenantServiceClient::connect(format!("http://{addr}")).await?;
        Ok(Self {
            container: None,
            client,
        })
    }
}
