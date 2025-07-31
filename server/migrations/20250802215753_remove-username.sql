-- Add migration script here
ALTER TABLE users
DROP COLUMN username,
ALTER first_name SET NOT NULL;
