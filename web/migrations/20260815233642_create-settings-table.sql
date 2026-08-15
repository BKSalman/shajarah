-- Add migration script here
CREATE TABLE IF NOT EXISTS settings (
    id BOOLEAN PRIMARY KEY DEFAULT TRUE,
    add_page_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT settings_singleton CHECK (id)
);

INSERT INTO settings (id) VALUES (TRUE) ON CONFLICT (id) DO NOTHING;
