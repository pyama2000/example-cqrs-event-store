variable "aws_service_endpoint" {
  description = "AWS service endpoint"
  type        = string
  default     = "http://localhost:4566"
}

variable "enable_integration" {
  description = "Enable integration service"
  type        = bool
  default     = false
}

variable "service_endpoints" {
  description = "Service endpoints"
  type = object({
    cart  = string,
    order = string,
  })
  default = null
}
