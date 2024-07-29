-- Add migration script here
DO $$ BEGIN
    CREATE TYPE UserRole AS ENUM (
        'admin', 'user'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;


CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    first_name TEXT,
    last_name TEXT,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    phone_number TEXT UNIQUE,
    password TEXT NOT NULL,
    role UserRole NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    last_login TIMESTAMPTZ
)
