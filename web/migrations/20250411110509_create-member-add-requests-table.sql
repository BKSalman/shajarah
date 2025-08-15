-- Add migration script here
DO $$ BEGIN
    CREATE TYPE request_status AS ENUM (
        'pending', 'approved', 'disapproved'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

CREATE TABLE IF NOT EXISTS member_add_requests
(
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    gender gender NOT NULL,
    birthday TIMESTAMPTZ,
    last_name TEXT NOT NULL,
    image BYTEA,
    image_type TEXT,
    mother_id INT8,
    father_id INT8,
    personal_info jsonb,
    status request_status NOT NULL DEFAULT 'pending',
    submitted_by TEXT,
    submitted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    reviewed_at TIMESTAMP,
    reviewed_by TEXT,
    rejection_reason TEXT,

   CONSTRAINT fk_mother
      FOREIGN KEY(mother_id)
        REFERENCES members(id)
        ON DELETE SET NULL,
   CONSTRAINT fk_father
      FOREIGN KEY(father_id)
        REFERENCES members(id)
        ON DELETE SET NULL
);
