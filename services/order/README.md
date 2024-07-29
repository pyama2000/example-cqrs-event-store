# order-service

## Requirements

- Docker Engine version: >= 24.0.0
- Docker Compose version: >= v2.28.0
- [aqua](https://github.com/aquaproj/aqua)
- Rust: 1.80.0
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
    - 注文の作成:

        ```bash
        curl -v "http://localhost:8080/orders" \
          -H "Content-Type: application/json" \
          -d '
        {
          "order": {
            "restaurant_id": "0190e34f-054f-7de1-890e-56d1f4bf0ccb",
            "user_id": "0190e34f-054f-7de1-890e-56d1f4bf0ccb",
            "delivery_address": "東京都"
          },
          "order_items": [{
            "item_id": "0190e34f-054f-7de1-890e-56d1f4bf0ccb", "price": 1000, "quantity": 5
          }]
        }'
        ```

    - 注文ステータスを「準備中」に変更する:

        ```bash
        curl -v "http://localhost:8080/orders/<注文ID>" \
          -H "Content-Type: application/json" \
          -d '{ "command": "prepare", "current_aggregate_version": 1 }'
        ```

    - 注文ステータスを「配達員のアサイン」に変更する:

        ```bash
        curl -v "http://localhost:8080/orders/<注文ID>" \
          -H "Content-Type: application/json" \
          -d '
        {
          "command": {
            "assigning_delivery_person": {
              "delivery_person_id": "0190e34f-054f-7de1-890e-56d1f4bf0ccb"
            },
          },
          "current_aggregate_version": 2
        }'
        ```

    - 注文ステータスを「準備完了」に変更する:

        ```bash
        curl -v "http://localhost:8080/orders/<注文ID>" \
          -H "Content-Type: application/json" \
          -d '{ "command": "ready_for_pickup", "current_aggregate_version": 3 }'
        ```

    - 注文ステータスを「配達員が商品を受け取った」に変更する:

        ```bash
        curl -v "http://localhost:8080/orders/<注文ID>" \
          -H "Content-Type: application/json" \
          -d '{ "command": "delivery_person_picking_up", "current_aggregate_version": 4 }'
        ```

    - 注文ステータスを「配達済み」に変更する:

        ```bash
        curl -v "http://localhost:8080/orders/<注文ID>" \
          -H "Content-Type: application/json" \
          -d '{ "command": "delivered", "current_aggregate_version": 5 }'
        ```

    - 注文ステータスを「キャンセル」に変更する:

        ```bash
        curl -v "http://localhost:8080/orders/<注文ID>" \
          -H "Content-Type: application/json" \
          -d '{ "command": "cancel", "current_aggregate_version": 5 }'
        ```

### Environment variables

環境変数を設定することで Docker Compose で起動するコンテナの設定を変えることができる

| Name | 説明 | デフォルト値 |
|-|-|-|
| LOCALSTACK_GATEWAY_PORT | LocalStack の Gateway のポート番号。AWS のサービスを操作するときに使用するポート。| 4566 |
| LOCALSTACK_EXTERNAM_SERVICE_PORT_RANGE | LocalStack の外部サービスのポート番号の範囲 | 4510-4559 |
| DOCKER_HOST_SOCK | LocalStack で AWS の一部サービスを Docker でエミュレートできるように、マウントするホストの Docker ソケットを指定する | /var/run/docker.sock |

## Testing

### Unit Test

> [!NOTE]
> Docker ホストを変えている場合は `DOCKER_HOST` 環境変数を設定する

```bash
cargo nextest --all-features --workspace
```

### Scenario Test

[runn](https://github.com/k1LoW/runn) でシナリオテストを実行できます。

```bash
runn run --verbose tests/runn/**/*.yaml
```
