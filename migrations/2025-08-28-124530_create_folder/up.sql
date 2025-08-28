-- Your SQL goes here
CREATE table folders (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,

    bucket_id UUID NOT NULL,
    parent_id UUID,
    created_by UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP
);

ALTER TABLE folders ADD UNIQUE (parent_id, name);
ALTER TABLE folders ADD FOREIGN KEY (parent_id) REFERENCES folders(id);
ALTER TABLE folders ADD FOREIGN KEY (bucket_id) REFERENCES buckets(id);
ALTER TABLE folders ADD FOREIGN KEY (created_by) REFERENCES users(id);
