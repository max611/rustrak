-- Active uptime monitoring — SQLite dialect. TEXT for UUIDs/timestamps.
-- See the Postgres migration of the same name for the full rationale.

CREATE TABLE uptime_monitors (
    id                   TEXT PRIMARY KEY,
    project_id           INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name                 TEXT NOT NULL,
    check_type           TEXT NOT NULL CHECK (check_type IN ('http', 'tcp')),
    url                  TEXT NOT NULL,
    interval_secs        INTEGER NOT NULL DEFAULT 60,
    timeout_secs         INTEGER NOT NULL DEFAULT 10,
    expected_status      INTEGER,
    fail_threshold       INTEGER NOT NULL DEFAULT 2,
    recovery_threshold   INTEGER NOT NULL DEFAULT 2,
    repeat_interval_secs INTEGER NOT NULL DEFAULT 3600,
    enabled              INTEGER NOT NULL DEFAULT 1,
    created_at           TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at           TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_uptime_monitors_project ON uptime_monitors(project_id);

CREATE TABLE uptime_checks (
    id            TEXT PRIMARY KEY,
    monitor_id    TEXT NOT NULL REFERENCES uptime_monitors(id) ON DELETE CASCADE,
    checked_at    TEXT NOT NULL DEFAULT (datetime('now')),
    status        INTEGER NOT NULL CHECK (status IN (0, 1, 2)),
    latency_ms    INTEGER,
    error_message TEXT
);

CREATE INDEX idx_uptime_checks_monitor_id ON uptime_checks(monitor_id);
CREATE INDEX idx_uptime_checks_checked_at ON uptime_checks(checked_at DESC);

CREATE TABLE uptime_monitor_states (
    monitor_id       TEXT PRIMARY KEY REFERENCES uptime_monitors(id) ON DELETE CASCADE,
    state            TEXT NOT NULL DEFAULT 'up' CHECK (state IN ('up', 'pending_down', 'down', 'pending_up')),
    fail_counter     INTEGER NOT NULL DEFAULT 0,
    recovery_counter INTEGER NOT NULL DEFAULT 0,
    last_check_at    TEXT,
    next_check_at    TEXT NOT NULL DEFAULT (datetime('now')),
    alerted_down_at  TEXT,
    last_alerted_at  TEXT,
    alert_count      INTEGER NOT NULL DEFAULT 0,
    incident_id      TEXT
);

CREATE TABLE uptime_incidents (
    id          TEXT PRIMARY KEY,
    monitor_id  TEXT NOT NULL REFERENCES uptime_monitors(id) ON DELETE CASCADE,
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE TABLE uptime_monitor_alert_channels (
    monitor_id     TEXT NOT NULL REFERENCES uptime_monitors(id) ON DELETE CASCADE,
    integration_id INTEGER NOT NULL REFERENCES alert_integrations(id) ON DELETE CASCADE,
    PRIMARY KEY (monitor_id, integration_id)
);
