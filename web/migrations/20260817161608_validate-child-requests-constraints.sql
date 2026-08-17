-- Add migration script here
ALTER TABLE member_add_requests VALIDATE CONSTRAINT fk_mother_request;
ALTER TABLE member_add_requests VALIDATE CONSTRAINT fk_father_request;
ALTER TABLE member_add_requests VALIDATE CONSTRAINT chk_not_self_mother;
ALTER TABLE member_add_requests VALIDATE CONSTRAINT chk_not_self_father;
ALTER TABLE member_add_requests VALIDATE CONSTRAINT chk_one_mother;
ALTER TABLE member_add_requests VALIDATE CONSTRAINT chk_one_father;
