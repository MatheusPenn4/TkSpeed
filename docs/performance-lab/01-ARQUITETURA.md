# 01 · Arquitetura — TkPerformanceLab

Novo **bounded context** (crate `tk-perflab`) que adiciona a capacidade de medição.
Reaproveita a infra existente (Clean Architecture/DDD/`AppContext`/SQLite) sem alterá-la.

## 1. Posição no workspace

```
tk-perflab  (NOVO)
 ├─ depende de: tk-contracts, tk-storage, tk-platform-win
 ├─ usado por:  src-tauri (bridge/comandos) e, no futuro, pelo TkOptimizer (Fase 2)
 └─ alimenta:   TkSpeed Score (subscore "Jogos"/"Estabilidade") e o Digital Twin

tk-platform-win  (ESTENDER — é o único lugar com acesso ao SO)
 └─ + adapters: PdhGpuCounters, EtwPresentTrace (FPS), LhmSensors (temp/clock/voltagem)
```

> O `SysinfoSampler` atual (tk-monitor) continua servindo o **dashboard ao vivo (1s, leve)**.
> O `tk-perflab` é **orientado a sessão** (alta fidelidade, só durante captura/benchmark).
> Não duplicamos coleta: os adapters de baixo nível ficam em `tk-platform-win`.

## 2. Componentes

### TkPerformanceLab (facade / casos de uso)
Orquestra: `capture_baseline()`, `run_benchmark(session_cfg)`, `compare(before, after)`,
`detect_bottlenecks(session)`. É o ponto único chamado pelo bridge.

### TkMetricsCollector (Strategy)
Trait + adapters; cada fonte declara `metrics()`, `cadence()`, `cost()`. Agrega num `MetricFrame`.
```
trait MetricsCollector {
    fn source(&self) -> CollectorId;
    fn start(&mut self) -> Result<()>;     // abre sessão (ex.: trace ETW)
    fn poll(&mut self) -> Vec<MetricPoint>; // amostra atual (sync, barato)
    fn stop(&mut self);                     // fecha sessão
}
```
Adapters: `SysinfoCollector` (CPU/RAM/disco), `PdhGpuCollector` (GPU%/VRAM), `EtwFpsCollector`
(present events → frametime/FPS), `LhmCollector` (temp/clock/voltagem — opcional).

### TkBenchmarkEngine
Executa uma **sessão de captura** por um período (gameplay real OU carga sintética), agrega o
buffer de frames/amostras e produz um `BenchmarkResult` (ver [03](03-BENCHMARK-COMPARACAO.md)).
Faz *warmup discard*, percentis e estatística de variância.

### TkComparisonEngine
Compara dois `BenchmarkResult` (ou dois `Baseline`) **com margem de erro**: classifica cada
métrica em **Ganho / Perda / Sem alteração (dentro do ruído)**. Nunca declara ganho não-significativo.

### Bottleneck Detectors
Sobre os frames de uma sessão, classificam CPU/GPU/RAM/Storage/Thermal bound (ver [04](04-DETECTORES.md)).

## 3. Modelo de dados (migração `0002_perflab.sql`)

```sql
CREATE TABLE perf_baselines (
  id            INTEGER PRIMARY KEY,
  ts            INTEGER NOT NULL,
  label         TEXT NOT NULL,            -- "antes da otimização X", "limpo de fábrica"...
  hardware_json TEXT NOT NULL,            -- HardwareInfo
  drivers_json  TEXT,                     -- versões de driver (GPU/chipset/rede/storage)
  score_total   INTEGER,                  -- TkSpeed Score no momento
  score_json    TEXT,                     -- breakdown
  context_json  TEXT NOT NULL             -- OS build, plano de energia, resolução...
);

CREATE TABLE benchmark_sessions (
  id            INTEGER PRIMARY KEY,
  baseline_id   INTEGER REFERENCES perf_baselines(id) ON DELETE SET NULL,
  ts            INTEGER NOT NULL,
  kind          TEXT NOT NULL,            -- game_capture | synthetic_cpu | synthetic_ram | synthetic_io
  suite_version TEXT NOT NULL,            -- comparabilidade só dentro da mesma versão
  duration_ms   INTEGER NOT NULL,
  target        TEXT,                     -- exe do jogo / nome do teste
  conditions_json TEXT NOT NULL,          -- resolução, preset, plano de energia (controle)
  runs          INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE benchmark_metrics (        -- 1 linha por métrica agregada por sessão
  id            INTEGER PRIMARY KEY,
  session_id    INTEGER NOT NULL REFERENCES benchmark_sessions(id) ON DELETE CASCADE,
  metric        TEXT NOT NULL,            -- fps_avg|fps_1pct_low|fps_01pct_low|frametime_p99|cpu_avg|gpu_avg|ram_avg|vram_avg|temp_max|clock_avg...
  value         REAL NOT NULL,
  unit          TEXT NOT NULL,
  stddev        REAL,                     -- desvio entre runs (base da margem de erro)
  samples       INTEGER NOT NULL
);

CREATE TABLE comparisons (
  id            INTEGER PRIMARY KEY,
  ts            INTEGER NOT NULL,
  before_session INTEGER NOT NULL REFERENCES benchmark_sessions(id),
  after_session  INTEGER NOT NULL REFERENCES benchmark_sessions(id),
  verdict_json  TEXT NOT NULL             -- por métrica: delta %, margem, classificação
);
```
Política de retenção integra o `housekeeping` existente (baselines/sessions mantidos por N e por idade).

## 4. Fluxo de dados

```
[Baseline ANTES]
  capture_baseline("antes") → HW + drivers + score + contexto → perf_baselines
        │
        ▼
[Benchmark ANTES]  (N runs, warmup descartado)
  run_benchmark(cfg) → MetricsCollector(start→poll…→stop) → buffer de frames/amostras
        → BenchmarkEngine.aggregate() → benchmark_sessions + benchmark_metrics
        → Detectors.classify() → veredito de gargalo
        │
        ▼
   [ usuário aplica mudança — FORA do escopo desta fase ]
        │
        ▼
[Benchmark DEPOIS]  (mesma suite_version, mesmas conditions)
        ▼
[Comparação]
  ComparisonEngine.compare(before, after) → por métrica: delta% ± margem
        → classifica Ganho/Perda/Sem alteração → comparisons → UI Performance Lab
```

## 5. Integração com IPC (futuro bridge)
Comandos planejados (a serem implementados no backlog): `perf_capture_baseline`,
`perf_run_benchmark`, `perf_compare`, `perf_list_baselines`, `perf_session_detail`.
Eventos de progresso/streaming de frametime: `perf:frame`, `perf:progress`.
Tipos novos vivem em `tk-contracts` (fonte única) — sem quebrar contratos existentes.
