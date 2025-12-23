-- Add migration script here
CREATE TABLE IF NOT EXISTS tree_invites
(
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);
