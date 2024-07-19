#trivy:ignore:AVD-AWS-0023 trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "aggregate" {
  name           = "restaurant-aggregate"
  billing_mode   = "PROVISIONED"
  read_capacity  = 20
  write_capacity = 20
  hash_key       = "id"

  attribute {
    name = "id"
    type = "S"
  }
}

#trivy:ignore:AVD-AWS-0023 trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "event_store" {
  name           = "restaurant-event"
  billing_mode   = "PROVISIONED"
  read_capacity  = 20
  write_capacity = 20
  hash_key       = "aggregate_id"
  range_key      = "id"

  attribute {
    name = "aggregate_id"
    type = "S"
  }

  attribute {
    name = "id"
    type = "S"
  }
}
