-- The platform a funnel event came from. NULL is a client older than the
-- field, which sent none.
ALTER TABLE anonymous_events ADD COLUMN platform TEXT;
