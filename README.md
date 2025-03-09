# Example of a CQRS event store

このリポジトリはAmazon Web Servicesブログの [Amazon DynamoDB を使った CQRS イベントストアの構築](https://aws.amazon.com/jp/blogs/news/build-a-cqrs-event-store-with-amazon-dynamodb/) を参考にRustでイベントソーシングを試してみたリポジトリです。

## Getting Started

1. RustアプリケーションのDockerイメージをビルドする
1. Docker Composeでコンテナを起動する

### RustアプリケーションのDockerイメージをビルドする

事前にRustアプリケーション (gRPCサーバー) のDockerイメージをビルドします。
事前にビルドするのは、Docker Composeでコンテナを起動する時間を短縮するためです。

> [!IMPORTANT]
> Dockerイメージのビルドに多くのリソースを利用するので1つずつビルドすることをおすすめします

```bash
# 各サービスを1つずつビルドする
docker buildx bake tenant \
  && docker buildx bake cart \
  && docker buildx bake order

# 各サービスを並列でビルドする
docker buildx bake
```

### Docker Composeでコンテナを起動する

Docker Composeでアプリケーションやデータストア (DynamoDB)、オブザーバビリティ関連のコンテナを起動します。
定義ファイルはルートディレクトリの `compose.yaml` と各アプリケーション ( `services` ディレクトリ配下) の `compose.yaml` を利用します。

> [!NOTE]
> アプリケーションの `compose.yaml` はルートディレクトリの `compose.override.yaml` で定義を一部書き換えています。
> 最終的に実行される定義ファイルは `docker compose config` (または `docker compose convert` ) を実行して確認できます。

起動されるコンテナの説明:

* **Grafana**:
  トレース・ログ・メトリクスを可視化します。
  `localhost:3000` でコンソールにアクセスできます。(任意のポートに変更したい場合は環境変数 `GRAFANA_PORT` に値を設定してください)
* **Grafana Tempo**: トレースデータを収集します
* **Grafana Loki**: ログデータを収集します
* **Prometheus**: メトリクスデータを収集します
* **OpenTelemetry Collector Agent**:
  アプリケーションから送信されたテレメトリーデータを収集し、バックエンド (OpenTelemetry Collector Gateway) に送信します。
  サービスの定義は各アプリケーション ( `services` ディレクトリ配下) の `compose.yaml` で定義されています。
* **OpenTelemetry Collector Gateway**: OpenTelemetry Collector Agent からテレメトリーデータを集約してバックエンドに送信します
* **LocalStack**:
  ローカルマシン上にAWS環境をエミュレートします。
  Amazon DynamoDBを利用します。
  ローカルからLocalStackにリクエストする場合は `localhost:4566` にアクセスしてください。(任意のポートに変更したい場合は環境変数 `LOCALSTACK_GATEWAY_PORT` に値を設定してください)
* **Terraform**:
  LocalStackに対してリソースを作成します。
  サービスの定義は各アプリケーション ( `services` ディレクトリ配下) の `compose.yaml` で定義されています。
* **アプリケーション**: gRPCサーバーを起動します
  * **テナントサービス**: `localhost:50051` でアクセスできます。(任意のポートに変更したい場合は環境変数 `TENANT_SERVICE_PORT` に値を設定してください)
  * **カートサービス**: `localhost:50052` でアクセスできます。(任意のポートに変更したい場合は環境変数 `CART_SERVICE_PORT` に値を設定してください)
  * **注文サービス**: `localhost:50053` でアクセスできます。(任意のポートに変更したい場合は環境変数 `ORDER_SERVICE_PORT` に値を設定してください)
