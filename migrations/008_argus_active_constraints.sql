-- argus_active_constraints table
-- Sentry's enforcement gate — promoted findings that block agent turns pre-flight.
-- When Sentry flags a CRITICAL/HIGH threat, she writes it here directly.
-- Every agent turn checks incoming messages against active constraints before
-- the LLM sees them. Matching constraints inject a hard warning block.

CREATE TABLE IF NOT EXISTS argus_active_constraints (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    constraint_name  TEXT UNIQUE NOT NULL,
    description      TEXT NOT NULL,
    pattern_keywords TEXT[] NOT NULL DEFAULT '{}',
    severity         TEXT NOT NULL DEFAULT 'HIGH',
    source_finding   TEXT,
    times_triggered  INT NOT NULL DEFAULT 0,
    created_by       TEXT NOT NULL DEFAULT 'argus-sentry',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_argus_constraints_severity
    ON argus_active_constraints (severity, times_triggered DESC);

-- RPC function: increment trigger count atomically
CREATE OR REPLACE FUNCTION increment_constraint_triggers(constraint_name TEXT)
RETURNS void AS $$
  UPDATE argus_active_constraints
  SET times_triggered = times_triggered + 1
  WHERE argus_active_constraints.constraint_name = increment_constraint_triggers.constraint_name;
$$ LANGUAGE sql;

-- RLS: service role only
ALTER TABLE argus_active_constraints ENABLE ROW LEVEL SECURITY;

DO $$ BEGIN
    CREATE POLICY "service_role_all" ON argus_active_constraints
        FOR ALL TO service_role USING (true) WITH CHECK (true);
EXCEPTION WHEN duplicate_object THEN null;
END $$;
