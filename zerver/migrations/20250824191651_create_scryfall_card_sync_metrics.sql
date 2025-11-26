CREATE TABLE scryfall_data_sync_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP,
    duration_in_seconds INT NOT NULL,
    status TEXT NOT NULL,
    received_count INT NOT NULL,
    upserted_count INT NOT NULL,
    skipped_count INT NOT NULL,
    error_count INT NOT NULL,
    errors JSONB NOT NULL DEFAULT '[]'::jsonb
);
