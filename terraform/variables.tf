variable "aws_service_endpoint" {
  description = "AWS service endpoint"
  type        = string
  default     = "http://localhost:4566"
}

variable "service_endpoints" {
  description = "Service endpoints"
  type = object({
    cart  = string,
    order = string,
  })
}
