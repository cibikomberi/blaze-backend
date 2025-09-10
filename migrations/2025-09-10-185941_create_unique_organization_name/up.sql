-- Your SQL goes here
ALTER TABLE organizations ADD CONSTRAINT unique_name UNIQUE (name);