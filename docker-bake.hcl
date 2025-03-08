group "default" {
  targets = [
    "tenant",
    "cart",
    "order",
  ]
}

target "_rust" {
  context    = "."
  dockerfile = "rust.Dockerfile"
}

target "tenant" {
  inherits = ["_rust"]
  tags     = ["tenant-service:latest"]
  args = {
    APPLICATION_NAME  = "tenant",
    SERVICE_DIRECTORY = "services/tenant",
  }
}

target "cart" {
  inherits = ["_rust"]
  tags     = ["cart-service:latest"]
  args = {
    APPLICATION_NAME  = "cart",
    SERVICE_DIRECTORY = "services/cart",
  }
}

target "order" {
  inherits = ["_rust"]
  tags     = ["order-service:latest"]
  args = {
    APPLICATION_NAME  = "order",
    SERVICE_DIRECTORY = "services/order",
  }
}
