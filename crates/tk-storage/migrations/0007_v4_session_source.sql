-- V4.1-E Session Source — adiciona source a profile_activations.
-- benchmark_sessions.source e optimization_runs.source já existem desde V4.1-A (0005).
-- Valores: 'manual' | 'profile_activation' | 'optimization_catalog' | 'automatic'
ALTER TABLE profile_activations ADD COLUMN source TEXT NOT NULL DEFAULT 'profile_activation';
