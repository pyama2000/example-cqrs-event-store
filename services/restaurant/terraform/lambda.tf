data "aws_iam_policy_document" "assume_role_lambda" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "event_router" {
  name               = "restaurant-service-event-router"
  assume_role_policy = data.aws_iam_policy_document.assume_role_lambda.json
}

data "archive_file" "dummy" {
  type        = "zip"
  output_path = "${path.module}/function.zip"

  source {
    content  = "dummy"
    filename = "bootstrap"
  }
}

#trivy:ignore:AVD-AWS-0066
resource "aws_lambda_function" "event_router" {
  function_name = "restaurant-service-event-router"
  role          = aws_iam_role.event_router.arn
  runtime       = "provided.al2023"
  architectures = ["arm64"]
  handler       = "bootstrap"
  filename      = data.archive_file.dummy.output_path

  environment {
    variables = {
      QUERY_MODEL_MAPPER_QUEUE_URL = aws_sqs_queue.query_model_mapper.url
    }
  }
}

resource "aws_lambda_event_source_mapping" "event_router" {
  event_source_arn  = aws_dynamodb_table.event_store.stream_arn
  function_name     = aws_lambda_function.event_router.arn
  starting_position = "LATEST"
}

resource "aws_iam_role" "query_model_mapper" {
  name               = "restaurant-service-query-model-mapper"
  assume_role_policy = data.aws_iam_policy_document.assume_role_lambda.json
}

#trivy:ignore:AVD-AWS-0066
resource "aws_lambda_function" "query_model_mapper" {
  function_name = "restaurant-service-query-model-mapper"
  role          = aws_iam_role.event_router.arn
  runtime       = "provided.al2023"
  architectures = ["arm64"]
  handler       = "bootstrap"
  filename      = data.archive_file.dummy.output_path

  environment {
    variables = {
      MYSQL_URL = "mysql://root:root@mysql:3306/query_model"
    }
  }
}

resource "aws_lambda_event_source_mapping" "query_model_mapper" {
  event_source_arn = aws_sqs_queue.query_model_mapper.arn
  function_name    = aws_lambda_function.query_model_mapper.arn
}
