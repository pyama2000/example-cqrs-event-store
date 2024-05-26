use std::future::Future;
use std::pin::Pin;

use adapter::persistence::connect;
use adapter::repository::WidgetRepository;
use app::WidgetServiceImpl;
use driver::Server;
use lib::{test_client, Error};
use reqwest::{RequestBuilder, Response, StatusCode};
use testcontainers::clients::Cli;
use testcontainers_modules::mysql::Mysql;

type AsyncAssertFn<'a> = fn(
    name: &'a str,
    response: Response,
) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

const WIDGET_NAME: &str = "部品名";
const WIDGET_DESCRIPTION: &str = "部品の説明";

/// 部品を作成するテスト
#[tokio::test]
async fn test_create_widget() -> Result<(), Error> {
    const ADDR: &str = "0.0.0.0:8081";

    let docker = Cli::default();
    let container = docker.run(Mysql::default());
    let pool = connect(&format!(
        "mysql://root@127.0.0.1:{}/mysql",
        container.get_host_port_ipv4(3306)
    ))
    .await?;
    sqlx::query(include_str!(
        "../migrations/20240210132634_create_aggregate.sql"
    ))
    .execute(&pool)
    .await?;
    sqlx::query(include_str!(
        "../migrations/20240210132646_create_event.sql"
    ))
    .execute(&pool)
    .await?;

    let repository = WidgetRepository::new(pool, test_client().await);
    let service = WidgetServiceImpl::new(repository);
    let server = Server::new(ADDR, service.into());
    tokio::spawn(async move { server.run().await.unwrap() });
    loop {
        let res = reqwest::get(format!("http://{ADDR}/healthz")).await;
        if matches!(res, Ok(res) if res.status().is_success()) {
            break;
        }
    }

    let client = reqwest::Client::new();
    struct TestCase<'a> {
        name: &'a str,
        request: RequestBuilder,
        assert: AsyncAssertFn<'a>,
    }
    let tests = vec![
        TestCase {
            name: "部品名・説明の形式が正しい場合、ボディが JSON で ID を含んで 201 が返る",
            request: client.clone().post(format!("http://{ADDR}/widgets")).json(
                &serde_json::json!({
                    "widget_name": WIDGET_NAME,
                    "widget_description": WIDGET_DESCRIPTION
                }),
            ),
            assert: (move |name, response: Response| {
                Box::pin(async move {
                    assert_eq!(response.status(), StatusCode::CREATED, "{name}");
                    assert!(
                        response
                            .json::<serde_json::Value>()
                            .await?
                            .get("widget_id")
                            .is_some(),
                        "{name}",
                    );
                    Ok(())
                })
            }),
        },
        TestCase {
            name: "部品名・説明の形式が不正な場合、400 が返る",
            request: client
                .clone()
                .post(format!("http://{ADDR}/widgets"))
                .json(&serde_json::json!({ "widget_name": "","widget_description": "" })),
            assert: (move |name, response: Response| {
                Box::pin(async move {
                    assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{name}");
                    Ok(())
                })
            }),
        },
    ];
    for test in tests {
        let res = test.request.send().await?;
        (test.assert)(test.name, res).await?;
    }
    Ok(())
}

/// 部品名を変更するテスト
#[tokio::test]
async fn test_change_widget_name() -> Result<(), Error> {
    const ADDR: &str = "0.0.0.0:8082";

    let docker = Cli::default();
    let container = docker.run(Mysql::default());
    let pool = connect(&format!(
        "mysql://root@127.0.0.1:{}/mysql",
        container.get_host_port_ipv4(3306)
    ))
    .await?;
    sqlx::query(include_str!(
        "../migrations/20240210132634_create_aggregate.sql"
    ))
    .execute(&pool)
    .await?;
    sqlx::query(include_str!(
        "../migrations/20240210132646_create_event.sql"
    ))
    .execute(&pool)
    .await?;

    let repository = WidgetRepository::new(pool, test_client().await);
    let service = WidgetServiceImpl::new(repository);
    let server = Server::new(ADDR, service.into());
    tokio::spawn(async move { server.run().await.unwrap() });
    loop {
        let res = reqwest::get(format!("http://{ADDR}/healthz")).await;
        if matches!(res, Ok(res) if res.status().is_success()) {
            break;
        }
    }

    struct TestCase {
        name: &'static str,
        widget_id: String,
        request: Box<dyn Fn(&str) -> RequestBuilder>,
        assert: fn(name: &str, response: Response),
    }
    let tests = vec![
        TestCase {
            name: "部品名の形式が正しい場合、202 が返る",
            widget_id: async {
                let res = reqwest::Client::new()
                    .post(format!("http://{ADDR}/widgets"))
                    .json(&serde_json::json!({
                        "widget_name": WIDGET_NAME,
                        "widget_description": WIDGET_DESCRIPTION
                    }))
                    .send()
                    .await?;
                let json: serde_json::Value = res.json::<serde_json::Value>().await?;
                Ok::<String, Error>(json.get("widget_id").unwrap().as_str().unwrap().to_string())
            }
            .await?,
            request: Box::new(|id| {
                reqwest::Client::new()
                    .post(format!("http://{ADDR}/widgets/{id}/name"))
                    .json(&serde_json::json!({ "widget_name": WIDGET_NAME }))
            }),
            assert: |name, response| {
                assert_eq!(response.status(), StatusCode::ACCEPTED, "{name}");
            },
        },
        TestCase {
            name: "部品名の形式が不正な場合、400 が返る",
            widget_id: async {
                let res = reqwest::Client::new()
                    .post(format!("http://{ADDR}/widgets"))
                    .json(&serde_json::json!({
                        "widget_name": WIDGET_NAME,
                        "widget_description": WIDGET_DESCRIPTION
                    }))
                    .send()
                    .await?;
                let json: serde_json::Value = res.json::<serde_json::Value>().await?;
                Ok::<String, Error>(json.get("widget_id").unwrap().as_str().unwrap().to_string())
            }
            .await?,
            request: Box::new(|id| {
                reqwest::Client::new()
                    .post(format!("http://{ADDR}/widgets/{id}/name"))
                    .json(&serde_json::json!({ "widget_name": "" }))
            }),
            assert: |name, response| {
                assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{name}");
            },
        },
    ];
    for test in tests {
        let res = (test.request)(&test.widget_id).send().await?;
        (test.assert)(test.name, res);
    }
    Ok(())
}

/// 部品の説明を変更するテスト
#[tokio::test]
async fn test_change_widget_description() -> Result<(), Error> {
    const ADDR: &str = "0.0.0.0:8083";

    let docker = Cli::default();
    let container = docker.run(Mysql::default());
    let pool = connect(&format!(
        "mysql://root@127.0.0.1:{}/mysql",
        container.get_host_port_ipv4(3306)
    ))
    .await?;
    sqlx::query(include_str!(
        "../migrations/20240210132634_create_aggregate.sql"
    ))
    .execute(&pool)
    .await?;
    sqlx::query(include_str!(
        "../migrations/20240210132646_create_event.sql"
    ))
    .execute(&pool)
    .await?;

    let repository = WidgetRepository::new(pool, test_client().await);
    let service = WidgetServiceImpl::new(repository);
    let server = Server::new(ADDR, service.into());
    tokio::spawn(async move { server.run().await.unwrap() });
    loop {
        let res = reqwest::get(format!("http://{ADDR}/healthz")).await;
        if matches!(res, Ok(res) if res.status().is_success()) {
            break;
        }
    }

    struct TestCase {
        name: &'static str,
        widget_id: String,
        request: Box<dyn Fn(&str) -> RequestBuilder>,
        assert: fn(name: &str, response: Response),
    }
    let tests = vec![
        TestCase {
            name: "部品の説明の形式が正しい場合、202 が返る",
            widget_id: async {
                let res = reqwest::Client::new()
                    .post(format!("http://{ADDR}/widgets"))
                    .json(&serde_json::json!({
                        "widget_name": WIDGET_NAME,
                        "widget_description": WIDGET_DESCRIPTION
                    }))
                    .send()
                    .await?;
                let json: serde_json::Value = res.json::<serde_json::Value>().await?;
                Ok::<String, Error>(json.get("widget_id").unwrap().as_str().unwrap().to_string())
            }
            .await?,
            request: Box::new(|id| {
                reqwest::Client::new()
                    .post(format!("http://{ADDR}/widgets/{id}/description"))
                    .json(&serde_json::json!({ "widget_description": WIDGET_DESCRIPTION }))
            }),
            assert: |name, response| {
                assert_eq!(response.status(), StatusCode::ACCEPTED, "{name}");
            },
        },
        TestCase {
            name: "部品の説明の形式が不正な場合、400 が返る",
            widget_id: async {
                let res = reqwest::Client::new()
                    .post(format!("http://{ADDR}/widgets"))
                    .json(&serde_json::json!({
                        "widget_name": WIDGET_NAME,
                        "widget_description": WIDGET_DESCRIPTION
                    }))
                    .send()
                    .await?;
                let json: serde_json::Value = res.json::<serde_json::Value>().await?;
                Ok::<String, Error>(json.get("widget_id").unwrap().as_str().unwrap().to_string())
            }
            .await?,
            request: Box::new(|id| {
                reqwest::Client::new()
                    .post(format!("http://{ADDR}/widgets/{id}/description"))
                    .json(&serde_json::json!({ "widget_description": "" }))
            }),
            assert: |name, response| {
                assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{name}");
            },
        },
    ];
    for test in tests {
        let res = (test.request)(&test.widget_id).send().await?;
        (test.assert)(test.name, res);
    }
    Ok(())
}
