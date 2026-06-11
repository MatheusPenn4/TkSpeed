# Fase 1 — MVP

**Meta da fase:** provar o ciclo *monitorar → diagnosticar → otimizar com rollback → medir* em hardware real, com fundação técnica sólida e zero risco de "quebrar o PC".
**Critério de saída:** aplicar uma otimização, medir o ganho e reverter 100% em 10+ máquinas.

Epics: E01 Plataforma · E02 Monitoramento · E03 Diagnóstico · E04 Score · E05 Benchmark · E06 Rollback · E07 Dashboard.

---

## TK-E01 · Plataforma & Persistência
> Fundação: bootstrap do core, SQLite, sistema de logs, IPC tipado, contratos e CI. **Bloqueia todo o resto.**

### TK-F011 · Bootstrap do TkCore & IPC tipado

#### TK-S0111 · Inicializar AppContext e ciclo de vida
- **Como** engenheiro, **quero** um `AppContext` que faça bootstrap (DB + event bus + módulos) **para** ter um ponto único de composição (DI).
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:**
  - `TK-T01111` Implementar `AppContext::bootstrap()` (abre DB, cria broadcast bus).
  - `TK-T01112` Lifecycle `bootstrap→ready→running→shutdown` com flush de logs/DB.
  - `TK-T01113` Registrar `AppContext` no `tauri::Builder.setup()` via `manage()`.
  - `TK-T01114` Teste de bootstrap (DB criado, bus ativo).
- **AC:** app inicia, cria `%APPDATA%\TkSpeed\tkspeed.db`, expõe contexto às telas; shutdown faz flush sem perda.
- **Dependências:** — (raiz)
- **DoD+:** cold start medido < 1.5 s.

#### TK-S0112 · Contratos compartilhados + geração de tipos (ts-rs)
- **Como** time full-stack, **quero** tipos únicos Rust→TS **para** eliminar drift no IPC.
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:**
  - `TK-T01121` Definir DTOs em `tk-contracts` (`MetricSample`, `Diagnosis`, `Finding`, `TkSpeedScore`, `Plan`, `PlanResult`).
  - `TK-T01122` `#[derive(TS)]` + task de build que exporta `.ts` para `apps/desktop/src/shared/ipc/types.ts`.
  - `TK-T01123` Wrappers `ipc.*` tipados no frontend.
  - `TK-T01124` Guard no CI: falhar se tipos gerados divergirem do commit.
- **AC:** alterar um struct Rust regenera o `.ts`; build quebra se houver drift.
- **Dependências:** TK-S0111

### TK-F012 · Camada de persistência SQLite

#### TK-S0121 · Conexão SQLite + migrations
- **Como** engenheiro, **quero** abrir o DB com WAL e rodar migrations **para** ter persistência transacional.
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:**
  - `TK-T01211` `tk_storage::open()` com WAL, `synchronous=NORMAL`, `foreign_keys=ON`.
  - `TK-T01212` Embutir e rodar `migrations/0001_init.sql` no boot.
  - `TK-T01213` Teste: DB novo aplica migration; reabrir é idempotente.
- **AC:** schema criado conforme [04-BANCO-DADOS](../04-BANCO-DADOS.md); reabrir não duplica.
- **Dependências:** TK-S0111

#### TK-S0122 · Repositórios base (Repository pattern)
- **Como** domínio, **quero** repos abstraídos por traits **para** não acoplar regras ao SQLite.
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🟢
- **Tasks:**
  - `TK-T01221` Traits `MetricRepository`, `ScoreRepository`, `AnalysisRepository`, `SnapshotRepository`, `AuditRepository`.
  - `TK-T01222` Implementações SQLite (sqlx) + mapeamento de erros.
  - `TK-T01223` Housekeeping noturno (downsampling/rollup + retenção + VACUUM).
- **AC:** CRUD coberto por testes de integração; rollup s1→s10→m1 funciona.
- **Dependências:** TK-S0121

### TK-F013 · Sistema de Logs & Telemetria de erro

#### TK-S0131 · Logging estruturado com rotação
- **Como** suporte/dev, **quero** logs JSON rotacionados **para** diagnosticar incidentes.
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:**
  - `TK-T01311` Configurar `tracing` + `tracing-subscriber` (JSON) → `%APPDATA%\TkSpeed\logs\`.
  - `TK-T01312` Rotação por tamanho/dia; níveis por módulo via env-filter.
  - `TK-T01313` Redaction de dados sensíveis (paths de usuário, ids).
  - `TK-T01314` `audit_log` append-only (helper `audit.log(action, details)`).
- **AC:** logs gravados, rotacionados, sem PII; auditoria registra ações de sistema.
- **Dependências:** TK-S0121
- **DoD+:** verificado que log não vaza dados sensíveis.

#### TK-S0132 · Crash handler (opt-in)
- **Prioridade:** P2 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** capturar panic→minidump local; UI de consentimento p/ envio; fila offline.
- **AC:** crash gera dump local; envio só com opt-in.
- **Dependências:** TK-S0131

### TK-F014 · Pipeline de CI/CD & health budget

#### TK-S0141 · CI Windows (build + test + lint + audit)
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** runner Windows; `cargo build/test/clippy`; `pnpm build/test`; `cargo audit`/`cargo deny`/`pnpm audit`; cache.
- **AC:** PR só funde com pipeline verde.
- **Dependências:** TK-S0111

#### TK-S0142 · Gate de health budget
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** teste que mede CPU idle/RAM do app monitorando; falha o build se exceder orçamento.
- **AC:** build falha se idle > 1% CPU ou > 150 MB RAM.
- **Dependências:** TK-S0141, TK-S0211

---

## TK-E02 · Monitoramento em Tempo Real
> TkMonitor: coleta assíncrona, event bus e streaming para a UI sem polling.

### TK-F021 · Coletores de telemetria (adapters)

#### TK-S0211 · Coletor base + cadências + backpressure
- **Como** usuário, **quero** métricas atualizando ao vivo **para** ver o estado do PC.
- **Prioridade:** P0 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:**
  - `TK-T02111` Trait `MetricCollector` (source, interval_ms, sample()).
  - `TK-T02112` Scheduler por fonte com cadências independentes (temp 1s, disco 2s, FPS 250ms).
  - `TK-T02113` Publicar `MetricSample` no broadcast com lag policy (descarta amostra velha).
  - `TK-T02114` Downsampler → persistência s1.
- **AC:** múltiplas fontes coletam em paralelo; overhead dentro do budget.
- **Dependências:** TK-S0111, TK-S0122

#### TK-S0212 · Adapter PDH (Performance Counters): CPU/RAM/Disco/Rede
- **Prioridade:** P0 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:** wrappers PDH em `tk-platform-win`; coletar uso CPU (total/por núcleo), RAM, IOPS/latência disco, throughput rede.
- **AC:** valores batem (±5%) com Gerenciador de Tarefas em cenários de teste.
- **Dependências:** TK-S0211

#### TK-S0213 · Adapter LibreHardwareMonitor: temp/clock/voltagem/GPU
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🔴
- **Tasks:** interop com LHM (avaliar licença — ver risco L3); coletar temperaturas, clocks, voltagem, GPU usage/VRAM; fallback se sensor ausente.
- **AC:** temperaturas/clocks aparecem em hardware suportado; degrada graciosamente onde não há sensor.
- **Dependências:** TK-S0211
- **DoD+:** auditoria de licença do componente registrada.

#### TK-S0214 · Stream de métricas para a UI (Tauri Channel)
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** comando `subscribe_metrics` via `ipc::Channel`; reemissão do bus; `useLiveMetrics` real (substitui mock).
- **AC:** UI recebe amostras em tempo real sem polling; desinscreve ao desmontar.
- **Dependências:** TK-S0211, TK-S0112

### TK-F022 · Inventário de hardware

#### TK-S0221 · Detecção e persistência de inventário
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** WMI para CPU/GPU/RAM/disco/MB; gravar `hardware_inventory` (first/last seen); detectar mudança de componente.
- **AC:** inventário correto na 1ª execução; troca de componente registra evento.
- **Dependências:** TK-S0122

---

## TK-E03 · Central de Diagnóstico
> TkAnalyzer: detecção de gargalo e auditores, com findings explicáveis.

### TK-F031 · Motor de gargalos

#### TK-S0311 · BottleneckEngine (CPU/RAM/Storage)
- **Como** usuário, **quero** saber meu gargalo **para** entender o que limita meu PC.
- **Prioridade:** P0 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:**
  - `TK-T03111` Janelas estatísticas sobre telemetria (média/percentil).
  - `TK-T03112` Regras CPU-bound, RAM-pressure, storage-bound → `Finding` (problema/impacto/solução).
  - `TK-T03113` Persistir `analysis_runs` + `findings`.
  - `TK-T03114` Testes unitários das regras com séries sintéticas.
- **AC:** cenário CPU-bound gera finding `High` com solução; tudo somente-leitura.
- **Dependências:** TK-S0211, TK-S0122

### TK-F032 · Auditor de inicialização

#### TK-S0321 · StartupAnalyzer
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** enumerar Run keys + Task Scheduler + Startup folder; estimar impacto no boot; finding por item pesado.
- **AC:** lista itens de boot com impacto estimado; nenhuma alteração feita.
- **Dependências:** TK-S0311

### TK-F033 · Orquestração & UI de diagnóstico

#### TK-S0331 · Comando analyze_full + progresso
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** `analyze_full` agrega auditores + score; evento `analyze:progress`; retorno `Diagnosis` tipado.
- **AC:** análise completa retorna findings + score; UI mostra progresso.
- **Dependências:** TK-S0311, TK-S0411

---

## TK-E04 · TkSpeed Score
> Modelo 0–1000 explicável (ver [13-TKSPEED-SCORE](../13-TKSPEED-SCORE.md)).

### TK-F041 · Motor de score

#### TK-S0411 · Cálculo de subscores e total
- **Prioridade:** P0 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:**
  - `TK-T04111` Normalização por categoria (CPU/GPU/RAM/Storage/Windows/Net/Temp/Jogos/Estabilidade).
  - `TK-T04112` Aplicar pesos versionados → total 0–1000 + classificação.
  - `TK-T04113` Persistir `scores` (breakdown_json + score_version).
  - `TK-T04114` Testes (máquina "perfeita"=Elite; faixas de classificação).
- **AC:** score reproduzível; breakdown coerente; versão registrada.
- **Dependências:** TK-S0311

#### TK-S0412 · Transparência do score (drill-down)
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** mapear cada subscore aos findings que o afetam; payload p/ UI.
- **AC:** clicar numa categoria mostra o que puxou o score.
- **Dependências:** TK-S0411, TK-S0331

---

## TK-E05 · Benchmark
> TkBenchmark: suites determinísticas + comparação antes/depois.

### TK-F051 · Suites de benchmark

#### TK-S0511 · Suites CPU/RAM/SSD
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:** CPU single/multi (compressão/cripto); RAM bandwidth/latência; SSD seq/rand; versionar `SUITE_VERSION`; persistir resultados.
- **AC:** resultados estáveis (variância < 5% em repetições na mesma máquina).
- **Dependências:** TK-S0122

#### TK-S0512 · Comparação antes/depois
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** comparar runs `before`/`after` da mesma versão; delta % por categoria; UI BeforeAfter.
- **AC:** delta correto; bloqueia comparação entre versões diferentes.
- **Dependências:** TK-S0511

---

## TK-E06 · Rollback
> TkRollback + saga: o pilar de confiança. **Bloqueia qualquer mutação de SO.**

### TK-F061 · Snapshots & restauração

#### TK-S0611 · SnapshotStore + integridade
- **Prioridade:** P0 · **Esforço:** L (8) · **Risco:** 🔴
- **Tasks:**
  - `TK-T06111` Implementar `SnapshotSink` sobre SQLite (`snapshots` + `snapshot_entries`).
  - `TK-T06112` Hash de integridade do snapshot; status active/restored/expired.
  - `TK-T06113` Restauração total e granular (por entrada) com validação de hash.
  - `TK-T06114` Retenção (mín. N snapshots) + expiração.
- **AC:** snapshot capturado e restaurado fielmente; hash inválido bloqueia restore.
- **Dependências:** TK-S0122
- **DoD+:** teste de restauração em VM.

#### TK-S0612 · Quarentena de arquivos
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** mover (não deletar) p/ quarentena com TTL; restaurar; expurgo pós-TTL.
- **AC:** arquivo "removido" é recuperável até o TTL.
- **Dependências:** TK-S0611

### TK-F062 · Saga transacional

#### TK-S0621 · Orquestrador de saga (commit/compensate)
- **Prioridade:** P0 · **Esforço:** L (8) · **Risco:** 🔴
- **Tasks:**
  - `TK-T06211` Implementar `run_plan` (snapshot→apply→verify→commit|compensate) ligando `SnapshotSink`/`AuditSink` reais.
  - `TK-T06212` Compensação em ordem inversa; idempotência do revert.
  - `TK-T06213` Recusar aplicação se snapshot falhar (invariante).
  - `TK-T06214` Testes de falha forçada → estado restaurado (VM).
- **AC:** falha no meio do plano reverte tudo; nunca fica estado parcial.
- **Dependências:** TK-S0611
- **DoD+:** teste em VM cobre falha em cada etapa.

### TK-F063 · UI de Rollback

#### TK-S0631 · Tela Rollback (timeline + diff + restaurar)
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** linha do tempo de snapshots; diff antes/depois; ações reverter tudo/item; quarentena.
- **AC:** usuário vê o que muda antes de restaurar; reverte total ou granular.
- **Dependências:** TK-S0611, TK-S0711

---

## TK-E07 · Dashboard & Design System UI
> Shell premium + telas do MVP. (Base já scaffolded.)

### TK-F071 · Design System & AppShell

#### TK-S0711 · Componentizar Design System (packages/ui)
- **Prioridade:** P0 · **Esforço:** L (8) · **Risco:** 🟢
- **Tasks:** extrair tokens + primitives (`GlassPanel`, `Button`, `MetricCard`, `ScoreGauge`, `RealtimeChart`, `Sparkline`); Storybook; `prefers-reduced-motion`.
- **AC:** componentes reutilizáveis documentados; acessibilidade AA.
- **Dependências:** TK-S0111

#### TK-S0712 · Janela custom Tauri (titlebar + controles reais)
- **Prioridade:** P1 · **Esforço:** S (2) · **Risco:** 🟢
- **Tasks:** ligar min/max/close à API da janela; região de drag.
- **AC:** controles de janela funcionam.
- **Dependências:** TK-S0711

### TK-F072 · Telas do MVP

#### TK-S0721 · Dashboard com dados reais
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** ligar gauge ao score real; cards à telemetria live; resumo de gargalos.
- **AC:** dashboard reflete dados reais em < 1.5 s; sem mock.
- **Dependências:** TK-S0214, TK-S0331, TK-S0411

#### TK-S0722 · Tela Monitoramento (gráficos tempo real)
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🟢
- **Tasks:** `RealtimeChart` (uPlot) por fonte; por-núcleo; export CSV.
- **AC:** gráficos fluidos (60fps), histórico de janela visível.
- **Dependências:** TK-S0214, TK-S0711

#### TK-S0723 · Tela Central de Diagnóstico
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** lista de findings com severidade/filtros; detalhe problema/impacto/solução; drill-down do score.
- **AC:** findings legíveis e filtráveis; ligados ao score.
- **Dependências:** TK-S0331, TK-S0412

#### TK-S0724 · Tela Benchmark
- **Prioridade:** P2 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** rodar suite com progresso; BeforeAfter; histórico.
- **AC:** roda benchmark e mostra antes/depois.
- **Dependências:** TK-S0512, TK-S0711
