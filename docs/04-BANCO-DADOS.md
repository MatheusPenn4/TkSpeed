# 04 · Banco de Dados (SQLite)

## Princípios

- **Local-first**, arquivo único `tkspeed.db` em `%APPDATA%\TkSpeed\`.
- **WAL** habilitado (concorrência leitura/escrita), `synchronous=NORMAL`.
- **Migrations versionadas** (`sqlx migrate`), nunca editadas após release.
- **Retenção**: telemetria de alta frequência é *downsampled*; rollups mantidos por política configurável.
- **Integridade**: snapshots têm hash; auditoria é append-only.

## Esquema lógico

```
machine_profile        (1)  → identidade da máquina (Digital Twin raiz)
hardware_inventory     (N)  → componentes detectados
metric_samples         (N)  → série temporal (rollups)
analysis_runs          (N)  → execuções de diagnóstico
findings               (N)  → gargalos/problemas por run
scores                 (N)  → TkSpeed Score por run
benchmark_runs         (N)  → execuções de benchmark
benchmark_results      (N)  → resultado por categoria
optimization_profiles  (N)  → perfis de otimização
optimizations_applied  (N)  → otimizações aplicadas (com snapshot ref)
snapshots              (N)  → estado salvo p/ rollback
snapshot_entries       (N)  → itens individuais do snapshot
game_profiles          (N)  → perfis de Game Boost por jogo
reports                (N)  → relatórios gerados
audit_log              (N)  → trilha append-only de tudo
settings               (k/v)→ preferências
license                (1)  → tier/ativação offline
schema_migrations      (N)  → controle de versão
```

## DDL (resumo — completo em `crates/tk-storage/migrations/`)

```sql
CREATE TABLE machine_profile (
  id            INTEGER PRIMARY KEY CHECK (id = 1),
  machine_uuid  TEXT NOT NULL,
  hostname      TEXT NOT NULL,
  os_version    TEXT NOT NULL,
  created_at    INTEGER NOT NULL,         -- epoch ms
  updated_at    INTEGER NOT NULL
);

CREATE TABLE hardware_inventory (
  id          INTEGER PRIMARY KEY,
  category    TEXT NOT NULL,              -- cpu|gpu|ram|storage|net|mb
  name        TEXT NOT NULL,
  details_json TEXT NOT NULL,
  first_seen  INTEGER NOT NULL,
  last_seen   INTEGER NOT NULL
);

CREATE TABLE metric_samples (
  id        INTEGER PRIMARY KEY,
  ts        INTEGER NOT NULL,            -- epoch ms (bucket)
  source    TEXT NOT NULL,               -- cpu|gpu|ram|ssd|net|temp...
  metric    TEXT NOT NULL,               -- usage|clock|temp|throughput|fps...
  value     REAL NOT NULL,
  unit      TEXT NOT NULL,
  rollup    TEXT NOT NULL                -- raw|s1|s10|m1
);
CREATE INDEX idx_metric_ts ON metric_samples(ts, source, metric);

CREATE TABLE analysis_runs (
  id          INTEGER PRIMARY KEY,
  started_at  INTEGER NOT NULL,
  finished_at INTEGER,
  trigger     TEXT NOT NULL,             -- manual|scheduled|gameboost
  summary_json TEXT
);

CREATE TABLE findings (
  id          INTEGER PRIMARY KEY,
  run_id      INTEGER NOT NULL REFERENCES analysis_runs(id),
  kind        TEXT NOT NULL,             -- bottleneck_cpu|driver_old|service|startup...
  severity    TEXT NOT NULL,             -- info|low|medium|high|critical
  title       TEXT NOT NULL,
  impact      TEXT NOT NULL,
  solution    TEXT NOT NULL,
  evidence_json TEXT
);

CREATE TABLE scores (
  id          INTEGER PRIMARY KEY,
  run_id      INTEGER REFERENCES analysis_runs(id),
  ts          INTEGER NOT NULL,
  total       INTEGER NOT NULL,          -- 0..1000
  classification TEXT NOT NULL,          -- critico|regular|bom|excelente|elite
  breakdown_json TEXT NOT NULL           -- por categoria
);

CREATE TABLE benchmark_runs (
  id          INTEGER PRIMARY KEY,
  ts          INTEGER NOT NULL,
  suite_version TEXT NOT NULL,
  context     TEXT                       -- before|after|standalone
);
CREATE TABLE benchmark_results (
  id          INTEGER PRIMARY KEY,
  run_id      INTEGER NOT NULL REFERENCES benchmark_runs(id),
  category    TEXT NOT NULL,             -- cpu_single|cpu_multi|ram|ssd|os
  raw_score   REAL NOT NULL,
  unit        TEXT NOT NULL
);

CREATE TABLE optimization_profiles (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL,
  description TEXT,
  items_json  TEXT NOT NULL,             -- lista de tweak ids + params
  is_builtin  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE snapshots (
  id          INTEGER PRIMARY KEY,
  ts          INTEGER NOT NULL,
  reason      TEXT NOT NULL,
  integrity_hash TEXT NOT NULL,
  status      TEXT NOT NULL              -- active|restored|expired
);
CREATE TABLE snapshot_entries (
  id          INTEGER PRIMARY KEY,
  snapshot_id INTEGER NOT NULL REFERENCES snapshots(id),
  target_type TEXT NOT NULL,             -- registry|service|power|file
  target_key  TEXT NOT NULL,
  old_value_json TEXT,                   -- estado anterior (p/ revert)
  new_value_json TEXT
);

CREATE TABLE optimizations_applied (
  id          INTEGER PRIMARY KEY,
  ts          INTEGER NOT NULL,
  tweak_id    TEXT NOT NULL,
  profile_id  INTEGER REFERENCES optimization_profiles(id),
  snapshot_id INTEGER NOT NULL REFERENCES snapshots(id),
  status      TEXT NOT NULL,             -- applied|reverted|failed
  result_json TEXT
);

CREATE TABLE game_profiles (
  id          INTEGER PRIMARY KEY,
  game_name   TEXT NOT NULL,
  exe_match   TEXT NOT NULL,
  config_json TEXT NOT NULL,             -- processos a suspender, power plan, etc.
  enabled     INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE reports (
  id          INTEGER PRIMARY KEY,
  ts          INTEGER NOT NULL,
  format      TEXT NOT NULL,             -- pdf|html
  path        TEXT NOT NULL,
  scope_json  TEXT
);

CREATE TABLE audit_log (
  id          INTEGER PRIMARY KEY,
  ts          INTEGER NOT NULL,
  actor       TEXT NOT NULL,             -- user|system|gameboost
  action      TEXT NOT NULL,
  details_json TEXT NOT NULL
);

CREATE TABLE settings (key TEXT PRIMARY KEY, value_json TEXT NOT NULL);
CREATE TABLE license (
  id INTEGER PRIMARY KEY CHECK (id=1),
  tier TEXT NOT NULL,                    -- free|pro|studio
  key_hash TEXT, activated_at INTEGER, expires_at INTEGER
);
```

## Retenção / housekeeping

| Rollup | Resolução | Retenção |
|---|---|---|
| `raw` | apenas em memória/live | não persistido |
| `s1` | 1s | 24 h |
| `s10` | 10s | 30 dias |
| `m1` | 1 min | 1 ano (Digital Twin) |

Tarefa de manutenção noturna agrega e expira conforme a política; `VACUUM` periódico.
