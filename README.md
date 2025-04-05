# Example of a CQRS event store

このリポジトリはAmazon Web Servicesブログの [Amazon DynamoDB を使った CQRS イベントストアの構築](https://aws.amazon.com/jp/blogs/news/build-a-cqrs-event-store-with-amazon-dynamodb/) を参考にRustでイベントソーシングを試してみたリポジトリです。

## Prerequisites

* Docker Compose: 2.22.0 and later

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
docker compose build tenant-service \
  && docker compose build cart-service \
  && docker compose build order-service

# 各サービスを並列でビルドする
docker compose build
```

### Docker Composeでコンテナを起動する

Docker Composeでアプリケーションやデータストア (DynamoDB)、オブザーバビリティ関連のコンテナを起動します。
定義ファイルはルートディレクトリの `compose.yaml` を利用します。

```bash
docker compose up

# アプリケーションコードの変更を検知してコンテナを作り直す
docker compose watch
```

起動されるコンテナの説明:

* **Grafana**:
  トレース・ログ・メトリクスを可視化します。
  `localhost:3000` でコンソールにアクセスできます。(任意のポートに変更したい場合は環境変数 `GRAFANA_PORT` に値を設定してください)
* **Grafana Tempo**: トレースデータを収集します
* **Grafana Loki**: ログデータを収集します
* **Prometheus**: メトリクスデータを収集します
* **OpenTelemetry Collector Agent**: アプリケーションから送信されたテレメトリーデータを収集し、バックエンド (OpenTelemetry Collector Gateway) に送信します
* **OpenTelemetry Collector Gateway**: OpenTelemetry Collector Agent からテレメトリーデータを集約してバックエンドに送信します
* **LocalStack**:
  ローカルマシン上にAWS環境をエミュレートします。
  ローカルからLocalStackにリクエストする場合は `localhost:4566` にアクセスしてください。(任意のポートに変更したい場合は環境変数 `LOCALSTACK_GATEWAY_PORT` に値を設定してください)
* **Terraform**:
  LocalStackに対してリソースを作成します。
  リソースの定義は `terraform` ディレクトリ配下で定義されています。
* **アプリケーション**: gRPCサーバーを起動します
  * **テナントサービス**: `localhost:50051` でアクセスできます。(任意のポートに変更したい場合は環境変数 `TENANT_SERVICE_PORT` に値を設定してください)
  * **カートサービス**: `localhost:50052` でアクセスできます。(任意のポートに変更したい場合は環境変数 `CART_SERVICE_PORT` に値を設定してください)
  * **注文サービス**: `localhost:50053` でアクセスできます。(任意のポートに変更したい場合は環境変数 `ORDER_SERVICE_PORT` に値を設定してください)

環境変数:

| 環境変数名 | 説明 | デフォルト値 |
|-|-|-|
| `GRAFANA_PORT` | Grafanaのコンソールにアクセスするためのポートを指定する | `3000` |
| `GRAFANA_TEMPO_LOG_LEVEL` | Grafana Tempoのログレベル | `error` |
| `LOCALSTACK_GATEWAY_PORT` | LocalStackにアクセスするためのポート | `4566` |
| `DOCKER_HOST_SOCK` | Dockerソケットの場所 | `-/var/run/docker.sock` |
| `TENANT_SERVICE_PORT` | テナントサービスにアクセスするためのポート | `50051` |
| `CART_SERVICE_PORT` | カートサービスにアクセスするためのポート | `50052` |
| `ORDER_SERVICE_PORT` | 注文サービスにアクセスするためのポート | `50053` |
