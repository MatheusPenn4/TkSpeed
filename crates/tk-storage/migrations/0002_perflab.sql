-- TkPerformanceLab · baselines, sessões de benchmark, métricas e comparações.

CREATE TABLE perf_baselines (
  id            INTEGER PRIMARY KEY,
  ts            INTEGER NOT NULL,
  label         TEXT NOT NULL,
  hardware_json TEXT NOT NULL DEFAULT '{}',
  drivers_json  TEXT,
  score_total   INTEGER,
  score_json    TEXT,
  context_json  TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE benchmark_sessions (
  id              INTEGER PRIMARY KEY,
  baseline_id     INTEGER REFERENCES perf_baselines(id) ON DELETE SET NULL,
  ts              INTEGER NOT NULL,
  kind            TEXT NOT NULL,            -- synthetic_cpu | synthetic_ram | synthetic_io | game_capture
  suite_version   TEXT NOT NULL,
  duration_ms     INTEGER NOT NULL,
  target          TEXT,                     -- rótulo do usuário / alvo
  conditions_json TEXT NOT NULL DEFAULT '{}',
  runs            INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX idx_bench_sessions_ts ON benchmark_sessions(ts);

CREATE TABLE benchmark_metrics (
  id          INTEGER PRIMARY KEY,
  session_id  INTEGER NOT NULL REFERENCES benchmark_sessions(id) ON DELETE CASCADE,
  metric      TEXT NOT NULL,
  value       REAL NOT NULL,
  unit        TEXT NOT NULL,
  stddev      REAL NOT NULL DEFAULT 0,
  samples     INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX idx_bench_metrics_session ON benchmark_metrics(session_id);

CREATE TABLE comparisons (
  id              INTEGER PRIMARY KEY,
  ts              INTEGER NOT NULL,
  before_session  INTEGER NOT NULL REFERENCES benchmark_sessions(id) ON DELETE CASCADE,
  after_session   INTEGER NOT NULL REFERENCES benchmark_sessions(id) ON DELETE CASCADE,
  verdict_json    TEXT NOT NULL
);
