-- Add migration script here
ALTER TABLE member_add_requests
    ADD COLUMN IF NOT EXISTS mother_request_id UUID,
    ADD COLUMN IF NOT EXISTS father_request_id UUID;

ALTER TABLE member_add_requests
    ADD CONSTRAINT fk_mother_request
        FOREIGN KEY (mother_request_id) REFERENCES member_add_requests(id)
        ON DELETE CASCADE NOT VALID,
    ADD CONSTRAINT fk_father_request
        FOREIGN KEY (father_request_id) REFERENCES member_add_requests(id)
        ON DELETE CASCADE NOT VALID,
    ADD CONSTRAINT chk_not_self_mother
        CHECK (mother_request_id IS DISTINCT FROM id) NOT VALID,
    ADD CONSTRAINT chk_not_self_father
        CHECK (father_request_id IS DISTINCT FROM id) NOT VALID,
    ADD CONSTRAINT chk_one_mother
        CHECK (num_nonnulls(mother_id, mother_request_id) <= 1) NOT VALID,
    ADD CONSTRAINT chk_one_father
        CHECK (num_nonnulls(father_id, father_request_id) <= 1) NOT VALID;

CREATE INDEX IF NOT EXISTS idx_mar_mother_request_id
    ON member_add_requests(mother_request_id);
CREATE INDEX IF NOT EXISTS idx_mar_father_request_id
    ON member_add_requests(father_request_id);
