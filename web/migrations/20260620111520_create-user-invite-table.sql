-- Add migration script here
CREATE TABLE IF NOT EXISTS user_invites (
    id BIGSERIAL PRIMARY KEY,
    token_hash   VARCHAR(255) NOT NULL UNIQUE,
    created_by   UUID NOT NULL REFERENCES users(id),
    expires_at   TIMESTAMPTZ,
    used_at      TIMESTAMPTZ,
    accepted_by  UUID REFERENCES users(id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
)
