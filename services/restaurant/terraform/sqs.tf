#trivy:ignore:AVD-AWS-0096
resource "aws_sqs_queue" "query_model_mapper" {
  name                        = "restaurant-service-query-model-mapper.fifo"
  fifo_queue                  = true
  content_based_deduplication = true
}
