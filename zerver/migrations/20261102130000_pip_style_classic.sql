-- The classic pip fills are stored as 'classic', no longer 'natural'.
-- zwipe-core's PipColor reads both names, so a row or payload written before
-- this still parses.

ALTER TABLE user_preferences
    DROP CONSTRAINT IF EXISTS user_preferences_pip_style_check;

UPDATE user_preferences SET pip_style = 'classic' WHERE pip_style = 'natural';

ALTER TABLE user_preferences
    ADD CONSTRAINT user_preferences_pip_style_check
        CHECK (pip_style IN ('theme', 'classic'));
