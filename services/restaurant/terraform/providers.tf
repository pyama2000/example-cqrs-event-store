provider "aws" {
  access_key                  = "test"
  secret_key                  = "test"
  region                      = "ap-northeast-1"
  s3_use_path_style           = true
  skip_credentials_validation = true
  skip_metadata_api_check     = true
  skip_requesting_account_id  = true

  endpoints {
    dynamodb = var.aws_service_endpoint
    iam      = var.aws_service_endpoint
    lambda   = var.aws_service_endpoint
    sqs      = var.aws_service_endpoint
  }
}

provider "archive" {}
