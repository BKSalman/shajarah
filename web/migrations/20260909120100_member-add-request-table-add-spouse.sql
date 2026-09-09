ALTER TABLE member_add_requests
    ADD COLUMN IF NOT EXISTS spouse_id INT8,
    ADD COLUMN IF NOT EXISTS spouse_of_request_id UUID,
    ADD COLUMN IF NOT EXISTS marriage_status marriage_status;

ALTER TABLE member_add_requests
    ADD CONSTRAINT fk_spouse
        FOREIGN KEY (spouse_id) REFERENCES members(id)
        ON DELETE SET NULL NOT VALID,
    ADD CONSTRAINT fk_spouse_request
        FOREIGN KEY (spouse_of_request_id) REFERENCES member_add_requests(id)
        ON DELETE CASCADE NOT VALID,
    ADD CONSTRAINT chk_not_self_spouse
        CHECK (spouse_of_request_id IS DISTINCT FROM id) NOT VALID,
    ADD CONSTRAINT chk_one_spouse
        CHECK (num_nonnulls(spouse_id, spouse_of_request_id) <= 1) NOT VALID,
    ADD CONSTRAINT chk_spouse_not_parent
        CHECK (spouse_id IS NULL
               OR (spouse_id IS DISTINCT FROM father_id
                   AND spouse_id IS DISTINCT FROM mother_id)) NOT VALID,
    -- A spouse always carries a state. Written in this direction on purpose:
    -- the reverse ("a state implies a spouse") would fail when fk_spouse nulls
    -- spouse_id on member deletion, and take the DELETE down with it.
    ADD CONSTRAINT chk_marriage_status_with_spouse
        CHECK (num_nonnulls(spouse_id, spouse_of_request_id) = 0
               OR marriage_status IS NOT NULL) NOT VALID;

CREATE INDEX IF NOT EXISTS idx_mar_spouse_of_request_id
    ON member_add_requests(spouse_of_request_id);
CREATE INDEX IF NOT EXISTS idx_mar_spouse_id
    ON member_add_requests(spouse_id);
