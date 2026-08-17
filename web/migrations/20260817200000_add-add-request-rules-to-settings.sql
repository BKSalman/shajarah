-- Admin-configurable required fields for the public add-request form.
-- diesel-guard:disable AddColumnCheck
ALTER TABLE settings
    ADD COLUMN IF NOT EXISTS add_request_rules jsonb NOT NULL DEFAULT '{}'::jsonb;
