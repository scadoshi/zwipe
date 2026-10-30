-- Accent-insensitive card search: "Jotun" finds "Jötun Grunt" and "Lim-Dul"
-- finds "Lim-Dûl the Necromancer". Additive: the extension and one function.
--
-- unaccent ships with Postgres contrib and is a trusted extension, so the
-- database owner can create it without superuser.
CREATE EXTENSION IF NOT EXISTS unaccent;

-- unaccent() is only STABLE because its dictionary could change. Naming the
-- dictionary pins it, which makes this wrapper safe to mark IMMUTABLE (usable
-- in an index expression if search ever needs one). zwipe-core's
-- accent_fold.rs mirrors these rules for the in-memory matcher.
CREATE OR REPLACE FUNCTION f_unaccent(text) RETURNS text
    LANGUAGE sql IMMUTABLE PARALLEL SAFE STRICT
    AS $$ SELECT public.unaccent('public.unaccent'::regdictionary, $1) $$;
