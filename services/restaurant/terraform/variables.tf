variable "aws_service_endpoint" {
  description = "AWS service endpoint"
  type        = string
  default     = "http://localhost:4566"
}

variable "should_migrate_dynamodb_table" {
  description = "DynamoDB のテーブルにアイテムをマイグレーションするかのフラグ。true ならマイグレーションする。"
  type        = bool
  default     = false
}
