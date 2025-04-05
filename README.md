# Example of a CQRS event store

このリポジトリは Amazon Web Services ブログの [Amazon DynamoDB を使った CQRS イベントストアの構築](https://aws.amazon.com/jp/blogs/news/build-a-cqrs-event-store-with-amazon-dynamodb/) を参考に Rust でイベントソーシングを試してみたリポジトリです。

## Example

### MySQL の DB を立ち上げてマイグレーションを実行する

```bash
docker compose up --wait
cargo run --bin migrate --features migrate
```

MySQL の設定は環境変数によって変更することができます。

| 環境変数名 | 説明 | デフォルト値 |
|:-|:-|:-|
| MYSQL_USERNAME | MySQL に接続するためのユーザー名 | `root` |
| MYSQL_PASSWORD | MySQL に接続するためのパスワード | `root` |
| MYSQL_PORT | ローカルにマッピングする MySQL のポート | `3306` |
| MYSQL_DATABASE | MySQL のデータベース名 | `widget` |

### サーバーを立ち上げる

```bash
cargo run --release
```

### MySQL クライアントで接続する

```bash
mysql -h 127.0.0.1 --user ${MYSQL_USERNAME:-root} -p${MYSQL_PASSWORD:-root} --port ${MYSQL_PORT:-3306} --database ${MYSQL_DATABASE:-widget}
```

### リクエスト

Widget を作成する:

```bash
curl -s http://localhost:8080/widgets \
  -H "Content-Type: application/json" \
  -d '{ "widget_name": "widget name 1", "widget_description": "widget description 1"}'
```

Widget の名前を変更する:

```bash
curl -v "http://localhost:8080/widgets/${WIDGET_ID}/name" \
  -H "Content-Type: application/json" \
  -d '{ "widget_name": "widget name 2"}'
```

Widget の説明を変更する:

```bash
curl -v "http://localhost:8080/widgets/${WIDGET_ID}/description" \
  -H "Content-Type: application/json" \
  -d '{ "widget_description": "widget description 2"}'
```
