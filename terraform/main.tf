#trivy:ignore:AVD-AWS-0023 trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "aggregate" {
  name           = "Aggregate"
  billing_mode   = "PROVISIONED"
  read_capacity  = 20
  write_capacity = 20
  hash_key       = "ID"

  attribute {
    name = "ID"
    type = "S"
  }
}

#trivy:ignore:AVD-AWS-0023 trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "event_store" {
  name           = "EventStore"
  billing_mode   = "PROVISIONED"
  read_capacity  = 20
  write_capacity = 20
  hash_key       = "AggregateID"
  range_key      = "ID"

  attribute {
    name = "AggregateID"
    type = "S"
  }

  attribute {
    name = "ID"
    type = "S"
  }
}
