# restaurant-service

## Requirements

- Docker Engine version: >= 24.0.0
- Docker Compose version: >= v2.28.0
- [aqua](https://github.com/aquaproj/aqua)
- Rust: 1.79.0
- (for testing) [cargo-nextest](https://github.com/nextest-rs/nextest)

## Getting Started

1. LocalStack のコンテナを Docker Compose で起動し、Terraform で DynamoDB のテーブルを作成する

    ```bash
    # バックグラウンドで実行する場合は --wait や -d/--detach オプションを利用する
    docker compose up
    # Lima を使っている場合は Docker ソケットをマウントするために lima コマンドを利用する
    ## マウントする Docker ソケットを確認する
    lima docker context inspect
    ## Lima でコンテナを起動する
    lima DOCKER_HOST_SOCK="<Docker ソケット>" docker compose up
    ```

2. API サーバーを起動する

    ```bash
    cargo run --release
    ```
3. API サーバーにリクエストする
    - 飲食店の作成:

        ```bash
        curl -v "http://localhost:8080/restaurants" \
            -H "Content-Type: application/json" \
            -d '{ "name": "テスト店舗" }'
        ```

    - 商品の追加:

        ```bash
        curl -v "http://localhost:8080/restaurants/<飲食店のID>/items" \
            -H "Content-Type: application/json" \
            -d '{
                "aggregate_version": 1,
                "items": [
                    {
                        "name": "テスト商品",
                        "price": { "Yen": 1000 },
                        "category": "Food"
                    }
                ]
            }'
        ```

    - 商品の削除:

        ```bash
        curl -v "http://localhost:8080/restaurants/<飲食店のID>/items" \
            -H "Content-Type: application/json" \
            -d '{
                "aggregate_version": 2,
                "item_ids": ["<商品のID>"]
            }'
        ```

### Environment variables

環境変数を設定することで Docker Compose で起動するコンテナのポートを変更することができる

| Name | 説明 | デフォルト値 |
|-|-|-|
| LOCALSTACK_GATEWAY_PORT | LocalStack の Gateway のポート番号。AWS のサービスを操作するときに使用するポート。| 4566 |
| LOCALSTACK_EXTERNAM_SERVICE_PORT_RANGE| LocalStack の外部サービスのポート番号の範囲 | 4510-4559 |
| DOCKER_HOST_SOCK | LocalStack で AWS の一部サービスを Docker でエミュレートできるように、マウントするホストの Docker ソケットを指定する | /var/run/docker.sock |

## Testing

### Unit Test

> [!NOTE]
> Docker ホストを変えている場合は `DOCKER_HOST` 環境変数を設定する必要がある

```bash
cargo nextest --all-features --workspace
```
