-- Your SQL goes here
CREATE TABLE organization_secrets (
    id CHAR(16) PRIMARY KEY NOT NULL,
    secret CHAR(32) NOT NULL ,
    organization_id UUID NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now()
);

ALTER TABLE organization_secrets ADD UNIQUE (organization_id, id);

ALTER TABLE organization_secrets ADD FOREIGN KEY (organization_id) REFERENCES organizations(id);
ALTER TABLE organization_secrets ADD FOREIGN KEY (created_by) REFERENCES users(id);