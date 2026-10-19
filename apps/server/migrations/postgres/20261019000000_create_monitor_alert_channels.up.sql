-- Alert-integration assignments for Sentry Crons monitors (BugWatch parity:
-- a missed/timed-out check-in now dispatches to Slack/webhook/email, mirroring
-- uptime_monitor_alert_channels for uptime monitors).

CREATE TABLE monitor_alert_channels (
    monitor_id     UUID NOT NULL REFERENCES monitors(id) ON DELETE CASCADE,
    integration_id INTEGER NOT NULL REFERENCES alert_integrations(id) ON DELETE CASCADE,
    PRIMARY KEY (monitor_id, integration_id)
);
