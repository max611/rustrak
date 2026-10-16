-- In-app notification inbox, created alongside outbound alert dispatch
-- (see services/alert.rs trigger_alert) for every project member — mirrors
-- BugWatch's noticed-gem-backed inbox (feature parity item).

CREATE TABLE notifications (
    id          SERIAL PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id  INTEGER REFERENCES projects(id) ON DELETE CASCADE,
    issue_id    UUID REFERENCES issues(id) ON DELETE CASCADE,
    kind        VARCHAR(32) NOT NULL,  -- 'new_issue' | 'regression' | 'unmute'
    title       VARCHAR(255) NOT NULL,
    body        TEXT NOT NULL DEFAULT '',
    url         TEXT,
    read_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_user_unread ON notifications(user_id, read_at, created_at DESC);
