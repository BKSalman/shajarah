-- Add migration script here
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX IF NOT EXISTS members_name_trgm_idx ON members USING GIN (name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS members_last_name_trgm_idx ON members USING GIN (last_name gin_trgm_ops);
