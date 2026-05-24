-- Migrate users on the removed "zwipe" theme to gruvbox (the new default).
UPDATE user_preferences SET theme = 'gruvbox' WHERE theme = 'zwipe';

-- Change column default so newly inserted rows pick up gruvbox.
ALTER TABLE user_preferences ALTER COLUMN theme SET DEFAULT 'gruvbox';
