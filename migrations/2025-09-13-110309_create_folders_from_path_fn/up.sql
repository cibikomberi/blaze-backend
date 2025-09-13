-- Your SQL goes here
CREATE OR REPLACE FUNCTION public.create_folders_for_path(p_bucket uuid, p_path text, p_created_by uuid)
    RETURNS uuid
    LANGUAGE plpgsql
AS $function$
DECLARE
    segments TEXT[];
    seg TEXT;
    curr_parent UUID;
    folder_id UUID;
BEGIN
    -- ensure root exists
    INSERT INTO folders (id, name, bucket_id, parent_id, created_by, created_at)
    VALUES (gen_random_uuid(), '', p_bucket, NULL, p_created_by, now())
    ON CONFLICT (bucket_id) WHERE parent_id IS NULL DO NOTHING;

    SELECT id INTO curr_parent
    FROM folders
    WHERE bucket_id = p_bucket AND parent_id IS NULL
    LIMIT 1;

    -- no path -> return root
    IF p_path IS NULL OR trim(p_path) = '' THEN
        RETURN curr_parent;
    END IF;

    -- split path into parts
    segments := array_remove(string_to_array(p_path, '/'), '');

    -- iterate and create hierarchy
    FOREACH seg IN ARRAY segments LOOP
            INSERT INTO folders (id, name, bucket_id, parent_id, created_by, created_at)
            VALUES (gen_random_uuid(), seg, p_bucket, curr_parent, p_created_by, now())
            ON CONFLICT (bucket_id, parent_id, name) DO NOTHING
            RETURNING id INTO folder_id;

            -- if insert skipped (conflict), fetch existing
            IF folder_id IS NULL THEN
                SELECT id INTO folder_id
                FROM folders
                WHERE bucket_id = p_bucket
                  AND parent_id = curr_parent
                  AND name = seg
                LIMIT 1;
            END IF;

            curr_parent := folder_id;
            folder_id := NULL;
        END LOOP;

    RETURN curr_parent; -- id of the last folder
END;
$function$
