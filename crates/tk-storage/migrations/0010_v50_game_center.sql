-- V5.0 Game Center — histórico de aplicação de perfis por jogo.
-- Registra qual perfil foi aplicado, quando, e para qual jogo.

CREATE TABLE IF NOT EXISTS game_runs (
    id         INTEGER PRIMARY KEY,
    ts         INTEGER NOT NULL,
    exe_match  TEXT    NOT NULL,
    game_name  TEXT    NOT NULL,
    profile_id TEXT    NOT NULL,
    source     TEXT    NOT NULL DEFAULT 'manual'
);
CREATE INDEX IF NOT EXISTS idx_game_runs_exe ON game_runs(exe_match, ts DESC);
