-- Add migration script here
CREATE TABLE IF NOT EXISTS system_metadata (
    key TEXT PRIMARY KEY,
    value NOT NULL jsonb,
)
