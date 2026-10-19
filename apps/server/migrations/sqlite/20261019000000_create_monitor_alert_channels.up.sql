CREATE TABLE monitor_alert_channels (
    monitor_id     TEXT NOT NULL REFERENCES monitors(id) ON DELETE CASCADE,
    integration_id INTEGER NOT NULL REFERENCES alert_integrations(id) ON DELETE CASCADE,
    PRIMARY KEY (monitor_id, integration_id)
);
