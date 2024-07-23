CREATE TABLE IF NOT EXISTS restaurant_item (
    aggregate_id VARCHAR(36) NOT NULL,
    item_id VARCHAR(36) NOT NULL,
    item_name TEXT NOT NULL,
    price INT NOT NULL,
    PRIMARY KEY (aggregate_id, item_id)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4;
