#trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "event_store" {
  name         = "tenant-event-store"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "aggregate_id"
  range_key    = "id"

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
resource "aws_dynamodb_table" "event_version" {
  name         = "tenant-event-version"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "aggregate_id"

  attribute {
    name = "aggregate_id"
    type = "S"
  }
}

#trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "aggregate" {
  name         = "tenant-aggregate"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "id"

  attribute {
    name = "id"
    type = "S"
  }
}
