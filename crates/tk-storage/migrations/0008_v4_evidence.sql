-- V4.1-F Evidence Engine Foundation — acumula evidência histórica por (fingerprint, config_id).
-- Cada registro responde: "quantas vezes esta config funcionou nesta máquina?"
CREATE TABLE config_evidence (
  id                    INTEGER PRIMARY KEY,
  fingerprint           TEXT    NOT NULL,
  config_id             TEXT    NOT NULL,
  source                TEXT    NOT NULL DEFAULT 'optimization_catalog',
  benchmark_relevance   TEXT    NOT NULL DEFAULT '[]',
  executions            INTEGER NOT NULL DEFAULT 0,
  successful_executions INTEGER NOT NULL DEFAULT 0,
  average_gain          REAL    NOT NULL DEFAULT 0.0,
  confidence            INTEGER NOT NULL DEFAULT 0,
  updated_at            INTEGER NOT NULL,
  UNIQUE (fingerprint, config_id)
);

CREATE INDEX idx_config_evidence_fingerprint ON config_evidence(fingerprint);
CREATE INDEX idx_config_evidence_config      ON config_evidence(config_id);
