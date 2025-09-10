-- Your SQL goes here
CREATE TYPE bucket_visibility AS ENUM ('private', 'public');

ALTER TABLE buckets ADD COLUMN visibility bucket_visibility NOT NULL DEFAULT 'private';