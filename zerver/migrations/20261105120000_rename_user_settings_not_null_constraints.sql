-- Postgres 18 names each NOT NULL constraint after its table, so the renamed
-- table still carries user_preferences_*_not_null. This renames every
-- constraint on user_settings that keeps the old prefix, on any version.

DO $$
DECLARE
    old_name TEXT;
BEGIN
    FOR old_name IN
        SELECT conname FROM pg_constraint
        WHERE conrelid = 'user_settings'::regclass
          AND conname LIKE 'user\_preferences\_%'
    LOOP
        EXECUTE format(
            'ALTER TABLE user_settings RENAME CONSTRAINT %I TO %I',
            old_name,
            'user_settings_' || substr(old_name, length('user_preferences_') + 1)
        );
    END LOOP;
END
$$;
