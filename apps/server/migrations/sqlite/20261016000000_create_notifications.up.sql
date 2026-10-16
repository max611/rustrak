-- In-app notification inbox — SQLite dialect. TEXT for timestamps/UUID.

CREATE TABLE notifications (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id  INTEGER REFERENCES projects(id) ON DELETE CASCADE,
    issue_id    TEXT REFERENCES issues(id) ON DELETE CASCADE,
    kind        VARCHAR(32) NOT NULL,
    title       VARCHAR(255) NOT NULL,
    body        TEXT NOT NULL DEFAULT '',
    url         TEXT,
    read_at     TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_notifications_user_unread ON notifications(user_id, read_at, created_at DESC);
