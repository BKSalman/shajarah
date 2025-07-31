-- Add migration script here
ALTER TABLE users
ADD totp_secret BYTEA,
ALTER password DROP NOT NULL;
