CREATE TABLE IF NOT EXISTS event (
    event_id VARCHAR(255) NOT NULL,
    widget_id VARCHAR(255) NOT NULL,
    event_name VARCHAR(255) NOT NULL,
    payload JSON NOT NULL,
    PRIMARY KEY (event_id),
    INDEX idx_widget_id (widget_id)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4;
