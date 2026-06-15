-- V4.2-B: profile_evidence — agrega execuções de perfis por (profile_id, fingerprint).
-- executions = total de ativações contabilizadas (Keep + Revert).
-- successful_executions = ativações com benchmark Keep confiável.
-- confidence = confidence_for_executions(successful_executions).

CREATE TABLE profile_evidence (
  id                    INTEGER PRIMARY KEY,
  profile_id            TEXT    NOT NULL,
  fingerprint           TEXT    NOT NULL,
  executions            INTEGER NOT NULL DEFAULT 0,
  successful_executions INTEGER NOT NULL DEFAULT 0,
  average_gain          REAL    NOT NULL DEFAULT 0.0,
  confidence            INTEGER NOT NULL DEFAULT 0,
  updated_at            INTEGER NOT NULL,
  UNIQUE (profile_id, fingerprint)
);

CREATE INDEX idx_profile_evidence_profile     ON profile_evidence(profile_id);
CREATE INDEX idx_profile_evidence_fingerprint ON profile_evidence(fingerprint);
