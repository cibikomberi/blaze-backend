-- This file should undo anything in `up.sql`
ALTER TABLE buckets DROP COLUMN visibility;

DROP TYPE bucket_visibility;