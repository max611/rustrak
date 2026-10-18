-- Active uptime monitoring (HTTP/TCP probing of external URLs) — distinct
-- from the Sentry Crons check-in monitoring already shipped (`monitors` /
-- `check_ins`, see 20261015120000_create_monitors). Ported from upstream's
-- closed, unmerged feat/uptime-monitoring-backend-core (PR #81): renamed
-- throughout to avoid colliding with those tables, given project scoping
-- (the original had none), and pointed at alert_integrations/integration_id
-- rather than the now-dead notification_channels/channel_id names that
-- predate this fork's alert_integrations rename (20260522200000).

CREATE TABLE uptime_monitors (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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
    enabled              BOOLEAN NOT NULL DEFAULT TRUE,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_uptime_monitors_project ON uptime_monitors(project_id);

CREATE TABLE uptime_checks (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    monitor_id    UUID NOT NULL REFERENCES uptime_monitors(id) ON DELETE CASCADE,
    checked_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status        INTEGER NOT NULL CHECK (status IN (0, 1, 2)),
    latency_ms    INTEGER,
    error_message TEXT
);

CREATE INDEX idx_uptime_checks_monitor_id ON uptime_checks(monitor_id);
CREATE INDEX idx_uptime_checks_checked_at ON uptime_checks(checked_at DESC);

CREATE TABLE uptime_monitor_states (
    monitor_id       UUID PRIMARY KEY REFERENCES uptime_monitors(id) ON DELETE CASCADE,
    state            TEXT NOT NULL DEFAULT 'up' CHECK (state IN ('up', 'pending_down', 'down', 'pending_up')),
    fail_counter     INTEGER NOT NULL DEFAULT 0,
    recovery_counter INTEGER NOT NULL DEFAULT 0,
    last_check_at    TIMESTAMPTZ,
    next_check_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    alerted_down_at  TIMESTAMPTZ,
    last_alerted_at  TIMESTAMPTZ,
    alert_count      INTEGER NOT NULL DEFAULT 0,
    incident_id      UUID
);

CREATE TABLE uptime_incidents (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    monitor_id  UUID NOT NULL REFERENCES uptime_monitors(id) ON DELETE CASCADE,
    started_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE TABLE uptime_monitor_alert_channels (
    monitor_id     UUID NOT NULL REFERENCES uptime_monitors(id) ON DELETE CASCADE,
    integration_id INTEGER NOT NULL REFERENCES alert_integrations(id) ON DELETE CASCADE,
    PRIMARY KEY (monitor_id, integration_id)
);
