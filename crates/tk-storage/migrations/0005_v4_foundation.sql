-- V4 Foundation · Schema base para Profile Engine, Evidence System e Machine Fingerprint.
-- Regra arquitetural: toda evidência é vinculada ao fingerprint da máquina.
-- Sessões de perfil não se misturam com sessões manuais (session_source).

-- ─────────────────────────────────────────────────────────────────────────────
-- 1. benchmark_sessions — colunas V4
-- ─────────────────────────────────────────────────────────────────────────────

-- Origem da sessão: separa Performance Lab de ativações de perfil.
-- Valores: 'manual' | 'profile_activation' | 'optimization_catalog' | 'automatic'
ALTER TABLE benchmark_sessions ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';

-- Fingerprint do hardware no momento do benchmark.
-- Formato: SHA-256(cpu_model + ":" + cpu_cores + ":" + gpu_name + ":" + ram_gb_rounded).
-- Nulo apenas em sessões antigas (pré-V4) sem fingerprint disponível.
ALTER TABLE benchmark_sessions ADD COLUMN machine_fingerprint TEXT;

-- GPU efetivamente usada durante o benchmark (nome do device de renderização).
-- Necessário para notebooks com iGPU + dGPU — evidências de devices diferentes não
-- podem ser acumuladas no mesmo bucket.
ALTER TABLE benchmark_sessions ADD COLUMN rendering_device_name TEXT;

-- ─────────────────────────────────────────────────────────────────────────────
-- 2. snapshots — colunas V4
-- ─────────────────────────────────────────────────────────────────────────────

-- Fingerprint do hardware no momento do snapshot (para arquivar evidência
-- de hardware trocado sem deletar dados históricos).
ALTER TABLE snapshots ADD COLUMN machine_fingerprint TEXT;

-- ─────────────────────────────────────────────────────────────────────────────
-- 3. optimization_runs — colunas V4
-- ─────────────────────────────────────────────────────────────────────────────

-- Origem da execução: separa catálogo individual de ativações de perfil.
ALTER TABLE optimization_runs ADD COLUMN source TEXT NOT NULL DEFAULT 'optimization_catalog';

-- ─────────────────────────────────────────────────────────────────────────────
-- 4. profile_definitions — definições de perfil (bundled + personalizados)
-- ─────────────────────────────────────────────────────────────────────────────

CREATE TABLE profile_definitions (
  id                TEXT    PRIMARY KEY,                  -- ex.: 'competitive', 'user_abc123'
  name              TEXT    NOT NULL,
  description       TEXT,
  icon              TEXT,                                 -- identificador de ícone Apex
  is_custom         INTEGER NOT NULL DEFAULT 0,           -- 0 = bundled, 1 = criado pelo usuário
  base_id           TEXT,                                 -- template base (somente se is_custom = 1)
  compositions_json TEXT    NOT NULL DEFAULT '[]',        -- [{config_id, value}, ...]
  suite_id          TEXT    NOT NULL DEFAULT 'complete',  -- suite de benchmark para evidência
  requires_fps      INTEGER NOT NULL DEFAULT 0,           -- exige FPS Measurement Ready
  bundle_version    INTEGER NOT NULL DEFAULT 0,           -- incrementado por release do app (bundled)
                                                          -- nunca alterado em perfis do usuário
  created_at        INTEGER NOT NULL,
  updated_at        INTEGER NOT NULL
);

-- ─────────────────────────────────────────────────────────────────────────────
-- 5. profile_state — perfil ativo por contexto de usuário
-- ─────────────────────────────────────────────────────────────────────────────
-- Usa user_context como PK (não singleton inteiro) para suportar múltiplos
-- contextos sem migração destrutiva futura.

CREATE TABLE profile_state (
  user_context   TEXT    NOT NULL DEFAULT 'default',
  profile_id     TEXT    REFERENCES profile_definitions(id) ON DELETE SET NULL,
  activated_at   INTEGER,
  snapshot_id    INTEGER REFERENCES snapshots(id) ON DELETE SET NULL,
  evidence_json  TEXT,                                    -- resultado do último benchmark comparison
  pending_reboot INTEGER NOT NULL DEFAULT 0,              -- 1 = alguma config aguarda reboot
  PRIMARY KEY (user_context)
);

-- Estado inicial: nenhum perfil ativo no contexto padrão.
INSERT INTO profile_state (user_context) VALUES ('default');

-- ─────────────────────────────────────────────────────────────────────────────
-- 6. profile_activations — histórico de ativações e evidências acumuladas
-- ─────────────────────────────────────────────────────────────────────────────

CREATE TABLE profile_activations (
  id                    INTEGER PRIMARY KEY,
  ts                    INTEGER NOT NULL,
  user_context          TEXT    NOT NULL DEFAULT 'default',
  profile_id            TEXT    NOT NULL,
  from_profile_id       TEXT,                              -- perfil que estava ativo antes
  snapshot_id           INTEGER NOT NULL REFERENCES snapshots(id),
  machine_fingerprint   TEXT,                              -- fingerprint no momento da ativação
  rendering_device_name TEXT,                              -- GPU primária no momento da ativação
  before_session_id     INTEGER REFERENCES benchmark_sessions(id) ON DELETE SET NULL,
  after_session_id      INTEGER REFERENCES benchmark_sessions(id) ON DELETE SET NULL,
  evidence_json         TEXT,                              -- PerfComparison serializado (nullable se sem medição)
  pending_reboot        INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_profile_activations_ts      ON profile_activations(ts DESC);
CREATE INDEX idx_profile_activations_profile ON profile_activations(profile_id, ts DESC);
CREATE INDEX idx_profile_activations_fp      ON profile_activations(machine_fingerprint, profile_id);
