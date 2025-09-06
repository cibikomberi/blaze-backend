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

ALTER TABLE folders ADD UNIQUE (bucket_id, parent_id, name);
CREATE UNIQUE INDEX unique_root_folder_per_bucket ON folders(bucket_id) WHERE parent_id IS NULL;

ALTER TABLE folders ADD FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE;
ALTER TABLE folders ADD FOREIGN KEY (bucket_id) REFERENCES buckets(id);
ALTER TABLE folders ADD FOREIGN KEY (created_by) REFERENCES users(id);
