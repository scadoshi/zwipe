-- The table and what is named after it take the settings name. The columns
-- stay as they are, so every query only changes the table it reads.

ALTER TABLE user_preferences RENAME TO user_settings;
ALTER TABLE user_settings RENAME CONSTRAINT user_preferences_pkey TO user_settings_pkey;
ALTER TABLE user_settings RENAME CONSTRAINT user_preferences_user_id_fkey TO user_settings_user_id_fkey;
ALTER TABLE user_settings RENAME CONSTRAINT user_preferences_pip_style_check TO user_settings_pip_style_check;
