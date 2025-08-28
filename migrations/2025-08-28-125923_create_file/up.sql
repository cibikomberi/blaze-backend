-- Your SQL goes here
CREATE table files (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    folder_id UUID NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP
);

ALTER TABLE files ADD UNIQUE (folder_id, name);

ALTER TABLE files ADD FOREIGN KEY (folder_id) REFERENCES folders(id);
ALTER TABLE files ADD FOREIGN KEY (created_by) REFERENCES users(id);