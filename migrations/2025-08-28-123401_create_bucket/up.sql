-- Your SQL goes here
CREATE TABLE buckets (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    organization_id UUID NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP
);

ALTER TABLE buckets ADD FOREIGN KEY (organization_id) REFERENCES organizations(id);
ALTER TABLE buckets ADD FOREIGN KEY (created_by) REFERENCES users(id);