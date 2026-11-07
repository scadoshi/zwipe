-- The pip style's first value is stored as `themed`, the name the code and UI
-- now use. The wire still writes `theme`, the only name 1.15.0 to 1.18.0
-- parse; context/plans/pip_themed_wire.md has the phases.

ALTER TABLE user_settings
    DROP CONSTRAINT IF EXISTS user_settings_pip_style_check;

UPDATE user_settings SET pip_style = 'themed' WHERE pip_style = 'theme';

ALTER TABLE user_settings
    ALTER COLUMN pip_style SET DEFAULT 'themed';

ALTER TABLE user_settings
    ADD CONSTRAINT user_settings_pip_style_check
        CHECK (pip_style IN ('themed', 'classic'));
