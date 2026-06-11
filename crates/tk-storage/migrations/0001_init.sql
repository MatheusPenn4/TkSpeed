-- TkSpeed · migration inicial
-- WAL e pragmas são aplicados na conexão (ver tk-storage/src/lib.rs).

CREATE TABLE machine_profile (
  id            INTEGER PRIMARY KEY CHECK (id = 1),
  machine_uuid  TEXT    NOT NULL,
  hostname      TEXT    NOT NULL,
  os_version    TEXT    NOT NULL,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL
);

CREATE TABLE hardware_inventory (
  id           INTEGER PRIMARY KEY,
  category     TEXT NOT NULL,
  name         TEXT NOT NULL,
  details_json TEXT NOT NULL,
  first_seen   INTEGER NOT NULL,
  last_seen    INTEGER NOT NULL
);

CREATE TABLE metric_samples (
  id     INTEGER PRIMARY KEY,
  ts     INTEGER NOT NULL,
  source TEXT NOT NULL,
  metric TEXT NOT NULL,
  value  REAL NOT NULL,
  unit   TEXT NOT NULL,
  rollup TEXT NOT NULL DEFAULT 's1'
);
CREATE INDEX idx_metric_ts ON metric_samples(ts, source, metric);

CREATE TABLE analysis_runs (
  id           INTEGER PRIMARY KEY,
  started_at   INTEGER NOT NULL,
  finished_at  INTEGER,
  trigger      TEXT NOT NULL,
  summary_json TEXT
);

CREATE TABLE findings (
  id            INTEGER PRIMARY KEY,
  run_id        INTEGER NOT NULL REFERENCES analysis_runs(id) ON DELETE CASCADE,
  kind          TEXT NOT NULL,
  severity      TEXT NOT NULL,
  title         TEXT NOT NULL,
  impact        TEXT NOT NULL,
  solution      TEXT NOT NULL,
  evidence_json TEXT
);

CREATE TABLE scores (
  id             INTEGER PRIMARY KEY,
  run_id         INTEGER REFERENCES analysis_runs(id) ON DELETE CASCADE,
  ts             INTEGER NOT NULL,
  total          INTEGER NOT NULL CHECK (total BETWEEN 0 AND 1000),
  classification TEXT NOT NULL,
  breakdown_json TEXT NOT NULL,
  score_version  TEXT NOT NULL
);

CREATE TABLE benchmark_runs (
  id            INTEGER PRIMARY KEY,
  ts            INTEGER NOT NULL,
  suite_version TEXT NOT NULL,
  context       TEXT
);
CREATE TABLE benchmark_results (
  id        INTEGER PRIMARY KEY,
  run_id    INTEGER NOT NULL REFERENCES benchmark_runs(id) ON DELETE CASCADE,
  category  TEXT NOT NULL,
  raw_score REAL NOT NULL,
  unit      TEXT NOT NULL
);

CREATE TABLE optimization_profiles (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL,
  description TEXT,
  items_json  TEXT NOT NULL,
  is_builtin  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE snapshots (
  id             INTEGER PRIMARY KEY,
  ts             INTEGER NOT NULL,
  reason         TEXT NOT NULL,
  integrity_hash TEXT NOT NULL,
  status         TEXT NOT NULL DEFAULT 'active'
);
CREATE TABLE snapshot_entries (
  id             INTEGER PRIMARY KEY,
  snapshot_id    INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
  target_type    TEXT NOT NULL,
  target_key     TEXT NOT NULL,
  old_value_json TEXT,
  new_value_json TEXT
);

CREATE TABLE optimizations_applied (
  id          INTEGER PRIMARY KEY,
  ts          INTEGER NOT NULL,
  tweak_id    TEXT NOT NULL,
  profile_id  INTEGER REFERENCES optimization_profiles(id),
  snapshot_id INTEGER NOT NULL REFERENCES snapshots(id),
  status      TEXT NOT NULL,
  result_json TEXT
);

CREATE TABLE game_profiles (
  id          INTEGER PRIMARY KEY,
  game_name   TEXT NOT NULL,
  exe_match   TEXT NOT NULL,
  config_json TEXT NOT NULL,
  enabled     INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE reports (
  id         INTEGER PRIMARY KEY,
  ts         INTEGER NOT NULL,
  format     TEXT NOT NULL,
  path       TEXT NOT NULL,
  scope_json TEXT
);

CREATE TABLE audit_log (
  id           INTEGER PRIMARY KEY,
  ts           INTEGER NOT NULL,
  actor        TEXT NOT NULL,
  action       TEXT NOT NULL,
  details_json TEXT NOT NULL
);

CREATE TABLE settings (
  key        TEXT PRIMARY KEY,
  value_json TEXT NOT NULL
);

CREATE TABLE license (
  id           INTEGER PRIMARY KEY CHECK (id = 1),
  tier         TEXT NOT NULL DEFAULT 'free',
  key_hash     TEXT,
  activated_at INTEGER,
  expires_at   INTEGER
);

INSERT INTO license (id, tier) VALUES (1, 'free');
