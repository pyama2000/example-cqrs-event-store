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

resource "aws_dynamodb_table_item" "aggregate" {
  count = var.should_migrate_dynamodb_table ? 1 : 0

  table_name = aws_dynamodb_table.aggregate.name
  hash_key   = aws_dynamodb_table.aggregate.hash_key
  item = jsonencode({
    "id" : {
      "S" : "0190d7b2-3940-7f92-9f86-ffe4cf69a5ed"
    }
    "version" : {
      "N" : "1"
    },
    "payload" : {
      "M" : {
        "V1" : {
          "M" : {
            "restaurant" : {
              "M" : {
                "V1" : {
                  "M" : {
                    "name" : {
                      "S" : "インドカレー 新宿店"
                    },
                  }
                }
              }
            },
            "items" : {
              "L" : [
                {
                  "M" : {
                    "V1" : {
                      "M" : {
                        "name" : {
                          "S" : "チキンカレー"
                        },
                        "id" : {
                          "S" : "0190d7b2-a2ca-7653-b005-244be7768ba4"
                        },
                        "category" : {
                          "S" : "Food"
                        },
                        "price" : {
                          "M" : {
                            "Yen" : {
                              "N" : "1000"
                            }
                          }
                        }
                      }
                    }
                  }
                }
              ]
            },
          }
        }
      }
    },
  })
}

#trivy:ignore:AVD-AWS-0023 trivy:ignore:AVD-AWS-0024 trivy:ignore:AVD-AWS-0025
resource "aws_dynamodb_table" "event_store" {
  name             = "restaurant-event"
  billing_mode     = "PROVISIONED"
  read_capacity    = 20
  write_capacity   = 20
  hash_key         = "aggregate_id"
  range_key        = "id"
  stream_enabled   = true
  stream_view_type = "NEW_IMAGE"

  attribute {
    name = "aggregate_id"
    type = "S"
  }

  attribute {
    name = "id"
    type = "S"
  }
}

resource "aws_dynamodb_table_item" "restaurant_created_event" {
  count = var.should_migrate_dynamodb_table ? 1 : 0

  table_name = aws_dynamodb_table.event_store.name
  hash_key   = aws_dynamodb_table.event_store.hash_key
  range_key  = aws_dynamodb_table.event_store.range_key
  item = jsonencode({
    "id" : {
      "S" : "0190d7b2-3940-7f92-9f87-001cf6d05516"
    },
    "aggregate_id" : {
      "S" : "0190d7b2-3940-7f92-9f86-ffe4cf69a5ed"
    },
    "payload" : {
      "M" : {
        "AggregateCreatedV1" : {
          "M" : {
            "V1" : {
              "M" : {
                "name" : {
                  "S" : "インドカレー 新宿店"
                },
              }
            }
          }
        }
      }
    },
  })
}

resource "aws_dynamodb_table_item" "item_added" {
  count = var.should_migrate_dynamodb_table ? 1 : 0

  table_name = aws_dynamodb_table.event_store.name
  hash_key   = aws_dynamodb_table.event_store.hash_key
  range_key  = aws_dynamodb_table.event_store.range_key
  item = jsonencode({
    "id" : {
      "S" : "0190d7b2-a2ca-7653-b005-245f55689fba"
    },
    "aggregate_id" : {
      "S" : "0190d7b2-3940-7f92-9f86-ffe4cf69a5ed"
    },
    "payload" : {
      "M" : {
        "ItemsAddedV1" : {
          "L" : [
            {
              "M" : {
                "V1" : {
                  "M" : {
                    "name" : {
                      "S" : "チキンカレー"
                    },
                    "id" : {
                      "S" : "0190d7b2-a2ca-7653-b005-244be7768ba4"
                    },
                    "category" : {
                      "S" : "Food"
                    },
                    "price" : {
                      "M" : {
                        "Yen" : {
                          "N" : "1000"
                        }
                      }
                    }
                  }
                }
              }
            }
          ]
        }
      }
    },
  })

  depends_on = [aws_dynamodb_table_item.restaurant_created_event]
}
