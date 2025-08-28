-- Your SQL goes here
CREATE TABLE organizations (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,

    created_by UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP
);

ALTER TABLE organizations ADD FOREIGN KEY (created_by) REFERENCES users(id);

CREATE TYPE organization_role as ENUM ('OWNER', 'ADMIN', 'EDITOR', 'COMMENTER', 'VIEWER');
CREATE TABLE user_organizations (
    user_id UUID NOT NULL,
    organization_id UUID NOT NULL,

    role organization_role NOT NULL DEFAULT 'VIEWER',
    added_by UUID,
    added_at TIMESTAMP NOT NULL DEFAULT now()
);

ALTER TABLE user_organizations ADD PRIMARY KEY (user_id, organization_id);
ALTER TABLE user_organizations ADD FOREIGN KEY (user_id) REFERENCES users(id);
ALTER TABLE user_organizations ADD FOREIGN KEY (organization_id) REFERENCES organizations(id);

ALTER TABLE user_organizations ADD FOREIGN KEY (added_by) REFERENCES users(id);