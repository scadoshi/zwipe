-- What the swipe screens say after a swipe: a quiet chip (the default), a
-- toast for each action, or nothing. Every row before the column, and every
-- client that does not send it, reads as 'chip'.

ALTER TABLE user_settings
    ADD COLUMN swipe_confirmations text NOT NULL DEFAULT 'chip'
        CONSTRAINT user_settings_swipe_confirmations_check
            CHECK (swipe_confirmations IN ('chip', 'toasts', 'off'));
