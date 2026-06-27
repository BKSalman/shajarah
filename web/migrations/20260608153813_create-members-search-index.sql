-- Add migration script here
-- requires pg_trgm extension
CREATE INDEX IF NOT EXISTS members_name_trgm_idx ON members USING GIN (name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS members_last_name_trgm_idx ON members USING GIN (last_name gin_trgm_ops);
