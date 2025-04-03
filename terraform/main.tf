locals {
  services = [
    {
      name = "tenant",
    },
    {
      name = "order",
    },
    {
      name = "cart",
      event_router = {
        environments = [
          {
            name  = "CART_SERVICE_ENDPOINT",
            value = var.service_endpoints["cart"],
          },
          {
            name  = "ORDER_SERVICE_ENDPOINT",
            value = var.service_endpoints["order"],
          },
        ],
      },
    },
  ]
}

###################
# Amazon DynamoDB #
###################
#trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "event_store" {
  for_each = { for service in local.services : service.name => service }

  name             = "${each.key}-event-store"
  billing_mode     = "PAY_PER_REQUEST"
  hash_key         = "aggregate_id"
  range_key        = "id"
  stream_enabled   = can(each.value.event_router) ? true : null
  stream_view_type = can(each.value.event_router) ? "NEW_IMAGE" : null

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
  for_each = { for service in local.services : service.name => service }

  name         = "${each.key}-event-sequence"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "aggregate_id"

  attribute {
    name = "aggregate_id"
    type = "S"
  }
}

#trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "aggregate" {
  for_each = { for service in local.services : service.name => service }

  name         = "${each.key}-aggregate"
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
  for_each = {
    for service in local.services : service.name => service
    if can(service.event_router)
  }

  name               = "${each.key}-event-router"
  assume_role_policy = data.aws_iam_policy_document.lambda_trust_relationship.json
}

resource "terraform_data" "dummy" {}

data "archive_file" "dummy" {
  type        = "zip"
  output_path = "${path.module}/function.zip"

  source {
    content  = "dummy"
    filename = "bootstrap"
  }

  depends_on = [terraform_data.dummy]
}

# Amazon DynamoDB Streamsのイベントを処理するAWS Lambda関数
#trivy:ignore:AVD-AWS-0066
resource "aws_lambda_function" "event_router" {
  for_each = {
    for service in local.services : service.name => service
    if can(service.event_router)
  }

  function_name = "${each.key}-event-router"
  role          = aws_iam_role.event_router[each.key].arn
  runtime       = "provided.al2023"
  architectures = ["arm64"]
  handler       = "bootstrap"
  filename      = data.archive_file.dummy.output_path

  environment {
    variables = { for env in each.value.event_router.environments : env.name => env.value }
  }

  lifecycle {
    ignore_changes = [
      runtime,
      filename,
    ]
  }
}

# Amazon DynamoDB StreamsとAWS Lambda関数を紐づける
resource "aws_lambda_event_source_mapping" "event_router" {
  for_each = {
    for service in local.services : service.name => service
    if can(service.event_router)
  }

  event_source_arn  = aws_dynamodb_table.event_store[each.key].stream_arn
  function_name     = aws_lambda_function.event_router[each.key].arn
  starting_position = "LATEST"
}
