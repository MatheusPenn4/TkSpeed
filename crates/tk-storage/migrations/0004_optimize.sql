-- TkOptimization Engine (Fase 2) · execuções do pipeline com evidência.
CREATE TABLE optimization_runs (
  id              INTEGER PRIMARY KEY,
  ts              INTEGER NOT NULL,
  optimization_id TEXT NOT NULL,
  snapshot_id     INTEGER NOT NULL REFERENCES snapshots(id),
  before_session  INTEGER REFERENCES benchmark_sessions(id) ON DELETE SET NULL,
  after_session   INTEGER REFERENCES benchmark_sessions(id) ON DELETE SET NULL,
  status          TEXT NOT NULL,          -- applied|kept|reverted|inconclusive|failed
  decision        TEXT NOT NULL,          -- keep|revert|inconclusive
  confidence      INTEGER NOT NULL DEFAULT 0,
  evidence_json   TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX idx_optruns_ts ON optimization_runs(ts);
