###################
# Amazon DynamoDB #
###################
#trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "event_store" {
  name             = "cart-event-store"
  billing_mode     = "PAY_PER_REQUEST"
  hash_key         = "aggregate_id"
  range_key        = "id"
  stream_enabled   = true
  stream_view_type = "NEW_IMAGE"

  attribute {
    name = "aggregate_id"
    type = "S"
  }

  attribute {
    name = "id"
    type = "N"
  }
}

#trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "event_sequence" {
  name         = "cart-event-sequence"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "aggregate_id"

  attribute {
    name = "aggregate_id"
    type = "S"
  }
}

#trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "aggregate" {
  name         = "cart-aggregate"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "id"

  attribute {
    name = "id"
    type = "S"
  }
}

##############
# AWS Lambda #
##############
data "aws_iam_policy_document" "lambda_trust_relationship" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "event_router" {
  name               = "cart-event-router"
  assume_role_policy = data.aws_iam_policy_document.lambda_trust_relationship.json
}

data "archive_file" "dummy" {
  type        = "zip"
  output_path = "${path.module}/function.zip"

  source {
    content  = "dummy"
    filename = "bootstrap"
  }
}

# Amazon DynamoDB Streamsのイベントを処理するAWS Lambda関数
#trivy:ignore:AVD-AWS-0066
resource "aws_lambda_function" "event_router" {
  function_name = "cart-event-router"
  role          = aws_iam_role.event_router.arn
  runtime       = "provided.al2023"
  architectures = ["arm64"]
  handler       = "bootstrap"
  filename      = data.archive_file.dummy.output_path
}

# Amazon DynamoDB StreamsとAWS Lambda関数を紐づける
resource "aws_lambda_event_source_mapping" "event_router" {
  event_source_arn  = aws_dynamodb_table.event_store.stream_arn
  function_name     = aws_lambda_function.event_router.arn
  starting_position = "LATEST"
}
