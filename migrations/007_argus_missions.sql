-- argus_missions table
-- Persists mission state across daemon restarts.
-- MissionRegistry upserts here on every status change.

CREATE TABLE IF NOT EXISTS argus_missions (
    id               UUID PRIMARY KEY,
    objective        TEXT NOT NULL,
    created_by       TEXT NOT NULL,
    primary_executor TEXT NOT NULL DEFAULT 'grok-build',
    status           TEXT NOT NULL DEFAULT 'planning',
    subtasks         JSONB NOT NULL DEFAULT '[]',
    deliverables     JSONB NOT NULL DEFAULT '[]',
    verification     JSONB NOT NULL DEFAULT '[]',
    sentry_request_id TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at     TIMESTAMPTZ
);

-- Index for fast active mission queries
CREATE INDEX IF NOT EXISTS idx_argus_missions_status
    ON argus_missions (status)
    WHERE status NOT IN ('complete', 'failed');

CREATE INDEX IF NOT EXISTS idx_argus_missions_created
    ON argus_missions (created_at DESC);

-- RLS: service role only — agents write via the daemon's service key
ALTER TABLE argus_missions ENABLE ROW LEVEL SECURITY;

CREATE POLICY "service_role_all" ON argus_missions
    FOR ALL TO service_role USING (true) WITH CHECK (true);
