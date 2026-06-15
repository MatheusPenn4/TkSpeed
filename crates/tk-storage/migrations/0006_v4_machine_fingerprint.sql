-- V4.1-D Machine Fingerprint — adiciona machine_fingerprint a optimization_runs.
-- benchmark_sessions e snapshots já receberam a coluna em 0005_v4_foundation.sql.
-- DEFAULT NULL garante que runs pré-V4 continuem funcionando.

ALTER TABLE optimization_runs ADD COLUMN machine_fingerprint TEXT;
