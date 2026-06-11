# 02 · Estrutura de Dados

## 1. Metadados de otimização (`OptimizationMeta`)
```rust
pub enum OptCategory { Cleanup, Energy, Startup, Services, Registry, Process, Memory, Scheduler, Storage }
pub enum RiskLevel { Safe, Moderate, Advanced, Experimental }
pub enum Reversibility { Full, FullViaQuarantine, RestorePointRecommended }

/// Como a otimização é validada (qual evidência prova/derruba o ganho).
pub enum Validation {
    Benchmark(String),     // suite a rodar antes/depois (ex.: "cpu-1.0.0", "fps-1.0.0")
    Score,                 // usa o TkSpeed Score antes/depois
    SpaceFreed,            // métrica não-perf: bytes liberados (limpezas)
    None,                  // informativo (ex.: só recomendação)
}

pub struct OptimizationMeta {
    pub id: String,                  // "cleanup.temp_files"
    pub name: String,
    pub description: String,
    pub category: OptCategory,
    pub risk: RiskLevel,
    pub reversibility: Reversibility,
    pub expected_impact: String,     // HONESTO (ex.: "libera espaço; não aumenta FPS")
    pub validation: Validation,      // benchmark associado / métrica de prova
    pub requires_elevation: bool,
    pub success_criteria: String,    // legível + avaliável (ex.: "ganho > margem e estável")
    pub rollback_criteria: String,   // quando reverter automaticamente
    pub enabled_by_default: bool,    // ADVANCED=false, EXPERIMENTAL=false e oculto
    pub hidden: bool,                // EXPERIMENTAL=true
}
```

## 2. Ação reversível (capturada no snapshot)
```rust
pub enum ReversibleAction {
    RegistryHkcu { subkey: String, name: String, old: Option<String>, new: Option<String> },
    FileQuarantine { path: String },                 // mover p/ quarentena (nunca deletar)
    PowerPlan { old_guid: String, new_guid: String },
    ServiceStart { name: String, old: String, new: String }, // StartType anterior/novo
}
```
Reusa o `snapshot_entries` (target_type/target_key/old/new) já existente — `ReversibleAction`
serializa para esse formato; o `ProtectionService` já sabe restaurar.

## 3. Execução do pipeline (`OptimizationRun`) + evidência
```rust
pub enum OptDecision { Keep, Revert, Inconclusive }

pub struct EvidenceRecord {
    pub score_before: Option<u16>,
    pub score_after: Option<u16>,
    pub before_session: Option<i64>,   // benchmark_sessions.id
    pub after_session: Option<i64>,
    pub comparison: Option<PerfComparison>, // do tk-perflab (ganho/perda/instável + confiança)
    pub space_freed_mb: Option<f64>,
    pub confidence: u8,
}

pub struct OptimizationRun {
    pub id: i64,
    pub ts: i64,
    pub optimization_id: String,
    pub profile_id: Option<i64>,
    pub snapshot_id: i64,
    pub status: String,        // applied | kept | reverted | failed | inconclusive
    pub decision: OptDecision,
    pub evidence: EvidenceRecord,
}
```

## 4. Recomendação do Advisor / Perfil
```rust
pub struct AdvisorRecommendation {
    pub optimization_id: String,
    pub bottleneck: BottleneckKind,  // do tk-perflab
    pub reason: String,
    pub expected_impact: String,
    pub risk: RiskLevel,
    pub prior_evidence: Option<String>, // "antes rendeu +6% em 5 runs" (histórico)
}

pub struct OptProfile {
    pub id: i64,
    pub name: String,                // "Competitive Gaming" ...
    pub description: String,
    pub category: String,
    pub items: Vec<String>,          // ids do catálogo (rastreável)
    pub is_builtin: bool,
}
```

## 5. Migração `0004_optimize.sql`
```sql
CREATE TABLE optimization_runs (
  id              INTEGER PRIMARY KEY,
  ts              INTEGER NOT NULL,
  optimization_id TEXT NOT NULL,
  profile_id      INTEGER REFERENCES optimization_profiles(id) ON DELETE SET NULL,
  snapshot_id     INTEGER NOT NULL REFERENCES snapshots(id),
  before_session  INTEGER REFERENCES benchmark_sessions(id) ON DELETE SET NULL,
  after_session   INTEGER REFERENCES benchmark_sessions(id) ON DELETE SET NULL,
  status          TEXT NOT NULL,          -- applied|kept|reverted|failed|inconclusive
  decision        TEXT NOT NULL,          -- keep|revert|inconclusive
  confidence      INTEGER NOT NULL DEFAULT 0,
  evidence_json   TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX idx_optruns_ts ON optimization_runs(ts);

CREATE TABLE optimization_profiles (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL,
  description TEXT,
  category    TEXT,
  items_json  TEXT NOT NULL DEFAULT '[]',
  is_builtin  INTEGER NOT NULL DEFAULT 1
);
```
> Reúso: snapshots/snapshot_entries (rollback), benchmark_sessions/benchmark_metrics
> (evidência), audit_log (auditoria) — **nada disso é recriado**.
> Retenção integra o `housekeeping` existente.

## 6. Honestidade de impacto (regra de modelagem)
`expected_impact` e a `Validation` **devem refletir a verdade**: uma limpeza de temporários
declara `Validation::SpaceFreed` e impacto "libera espaço" — **nunca** um ganho de FPS fabricado.
Otimizações de CPU usam `Benchmark("cpu-1.0.0")`; de jogo, `Benchmark("fps-1.0.0")`. Se a
otimização não tem como ser medida em performance, a evidência é o que ela realmente faz.
