-- Project-scoped API keys: a single secret that authenticates a project's
-- own read/write access to the /api/v1/* surface, mirroring BugWatch's
-- per-project api_key (feature parity item). Distinct from the DSN's
-- sentry_key (ingestion only) and from auth_tokens (user-scoped, not
-- project-scoped).

CREATE TABLE project_api_keys (
    id            SERIAL PRIMARY KEY,
    project_id    INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    key           VARCHAR(44) NOT NULL UNIQUE,  -- "rpk_" + 40 lowercase hex
    description   TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at  TIMESTAMPTZ
);

CREATE INDEX idx_project_api_keys_project_id ON project_api_keys(project_id);
