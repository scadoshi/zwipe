-- Backfill users.first_swiped_at for users who swiped before tracking existed.
-- Two sources, take the earliest:
--   - deck_cards.created_at: exact — a card in a deck means a right swipe happened
--   - user_daily_activity.day: day precision — catches swipes that never added a
--     card (left/up/down only); represented as midnight UTC of that day
-- Without this, veterans would fire a bogus today-dated first_swipe event on
-- their next usage flush.
UPDATE users u
SET first_swiped_at = sub.first_swipe
FROM (
    SELECT user_id, MIN(first_seen) AS first_swipe
    FROM (
        SELECT user_id, day::timestamptz AS first_seen
        FROM user_daily_activity
        WHERE swipes_right + swipes_left + swipes_up + swipes_down > 0
        UNION ALL
        SELECT d.user_id, dc.created_at
        FROM deck_cards dc
        JOIN decks d ON d.id = dc.deck_id
    ) x
    GROUP BY user_id
) sub
WHERE u.id = sub.user_id AND u.first_swiped_at IS NULL;
