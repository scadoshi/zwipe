CREATE TABLE scryfall_data_sync_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sync_type TEXT NOT NULL,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP,
    duration_in_seconds INT NOT NULL,
    status TEXT NOT NULL,
    received INT NOT NULL,
    imported INT NOT NULL,
    skipped INT NOT NULL,
    error_count INT NOT NULL,
    errors JSONB NOT NULL DEFAULT '[]'::jsonb
);