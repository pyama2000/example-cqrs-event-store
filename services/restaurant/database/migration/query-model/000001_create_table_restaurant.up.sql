CREATE TABLE IF NOT EXISTS restaurant (
    aggregate_id VARCHAR(36) NOT NULL,
    restaurant_name TEXT NOT NULL,
    PRIMARY KEY (aggregate_id)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4;
