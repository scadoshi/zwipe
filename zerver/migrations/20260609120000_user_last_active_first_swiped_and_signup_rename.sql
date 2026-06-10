ALTER TABLE users
    ADD COLUMN last_active_at TIMESTAMPTZ,
    ADD COLUMN first_swiped_at TIMESTAMPTZ;

UPDATE user_events SET kind = 'register' WHERE kind = 'signup';
