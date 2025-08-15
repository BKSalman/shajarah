-- Add migration script here
DO $$ BEGIN
    CREATE TYPE invite_status AS ENUM (
        'pending', 'accepted', 'declined'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

CREATE TABLE IF NOT EXISTS member_invites
(
    id UUID PRIMARY KEY,
    member_id BIGSERIAL NOT NULL,
    email TEXT NOT NULL,
    status invite_status NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    totp_secret BYTEA,

   CONSTRAINT fk_member
      FOREIGN KEY(member_id)
        REFERENCES members(id)
        ON DELETE SET NULL
);
