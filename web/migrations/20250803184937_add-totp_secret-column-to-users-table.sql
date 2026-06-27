-- Add migration script here
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_secret BYTEA;
