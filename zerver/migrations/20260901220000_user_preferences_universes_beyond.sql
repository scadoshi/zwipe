-- Global Universes Beyond preference (context/plans/in_universe_filter.md
-- step 5, owner 2026-09-01): exclude_universes_beyond hides cards with no
-- in-universe printing from the deck-aware serve and commander select;
-- universes_beyond_exceptions holds franchise slugs (zwipe-core
-- universe::FRANCHISES) whose cards are served anyway ("exclude UB but let
-- LotR through"). Additive: defaults leave every existing user unchanged, and
-- the COALESCE-based partial update means old clients can't wipe the fields.

ALTER TABLE user_preferences
    ADD COLUMN exclude_universes_beyond boolean NOT NULL DEFAULT false,
    ADD COLUMN universes_beyond_exceptions text[] NOT NULL DEFAULT '{}';
