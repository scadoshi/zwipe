-- Pip style preference: whether mana pips draw in the theme's own mana
-- colors ('theme') or in Magic's classic fills ('natural'). zwipe-core
-- PipStyle names the two values. Additive: the default keeps every existing
-- user on the theme colors, and the COALESCE-based partial update means old
-- clients can't wipe the field.

ALTER TABLE user_preferences
    ADD COLUMN pip_style text NOT NULL DEFAULT 'theme'
        CHECK (pip_style IN ('theme', 'natural'));
