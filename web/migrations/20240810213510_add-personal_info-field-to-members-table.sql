-- Add migration script here
ALTER TABLE members
ADD IF NOT EXISTS personal_info jsonb;
