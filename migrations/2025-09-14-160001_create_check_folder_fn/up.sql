-- Your SQL goes here
CREATE OR REPLACE FUNCTION public.folder_exists_for_path(
    p_bucket uuid,
    p_path text
) RETURNS uuid
    LANGUAGE plpgsql AS $function$
DECLARE
    segments TEXT[];
    seg TEXT;
    curr_parent UUID;
BEGIN
    -- root
    SELECT id INTO curr_parent
    FROM folders
    WHERE bucket_id = p_bucket AND parent_id IS NULL
    LIMIT 1;

    IF p_path IS NULL OR trim(p_path) = '' THEN
        RETURN curr_parent;
    END IF;

    segments := array_remove(string_to_array(p_path, '/'), '');

    FOREACH seg IN ARRAY segments LOOP
            SELECT id INTO curr_parent
            FROM folders
            WHERE bucket_id = p_bucket AND parent_id = curr_parent AND name = seg
            LIMIT 1;

            IF curr_parent IS NULL THEN
                RETURN NULL; -- folder missing
            END IF;
        END LOOP;

    RETURN curr_parent;
END;
$function$;
