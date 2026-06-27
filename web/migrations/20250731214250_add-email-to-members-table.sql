-- Add migration script here
ALTER TABLE members ADD COLUMN IF NOT EXISTS email TEXT;
