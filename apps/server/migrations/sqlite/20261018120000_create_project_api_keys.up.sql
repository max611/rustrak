-- Project-scoped API keys — SQLite dialect.

CREATE TABLE project_api_keys (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id    INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    key           VARCHAR(44) NOT NULL UNIQUE,
    description   TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    last_used_at  TEXT
);

CREATE INDEX idx_project_api_keys_project_id ON project_api_keys(project_id);
