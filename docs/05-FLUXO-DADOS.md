# 05 · Fluxo de Dados

## 1. Telemetria em tempo real (alta frequência)

```
[Sensores HW] → Adapters (LHM/WMI/PDH/ETW)
      │ amostra bruta
      ▼
[TkMonitor Sampler] (cadência por fonte)
      │ MetricSample (normalizada)
      ├──────────────▶ EventBus (broadcast Tokio)
      │                    │
      │                    ▼
      │             [Tauri Channel] ── evento "metrics:sample" ──▶ [React useMetricsStream]
      │                                                                  │
      │                                                                  ▼
      │                                                        Charts / Gauges (60fps render)
      ▼
[Downsampler] (1s/10s/1m rollups)
      ▼
[tk-storage] → tabela metric_samples (retenção por política) → Digital Twin
```

- UI assina o canal e renderiza; **não há polling** do frontend.
- Persistência é *downsampled* (não grava todas as amostras de 250ms) para conter o tamanho do DB.

## 2. Análise / Diagnóstico (Query — CQRS leitura)

```
[UI: "Analisar"] ─ invoke("analyze_full") ─▶ [analyze_cmd]
      ▼
[TkAnalyzer] lê janela de telemetria (storage + live) + drivers + serviços + startup
      ▼
BottleneckEngine + Auditores → Diagnosis
      ▼
TkSpeedScore.compute(Diagnosis, Benchmark) → score 0–1000
      ▼
retorna Diagnosis+Score (tipado) ─▶ UI renderiza Central de Diagnóstico
            └─ progresso por evento "analyze:progress"
```

## 3. Otimização (Command — CQRS escrita, transacional)

```
[UI: aplicar plano] ─ invoke("optimize_apply", plan) ─▶ [optimize_cmd]
      ▼
PermissionBroker → precisa elevar? → UAC (se sim)
      ▼
OptimizationSaga:
   1. snapshot()  ── TkRollback grava estado atual (Registry/serviços/power/arquivos→quarentena)
   2. log(intent) ── auditoria append-only
   3. apply(tx)   ── cada Optimization aplica seu tweak
   4. verify()    ── checa pós-condições
   5a. commit()   ── persiste resultado + emite "optimize:done"
   5b. compensate(snapshot) ── em qualquer falha, REVERTE tudo e emite "optimize:rolledback"
      ▼
UI atualiza estado + oferece "comparar antes/depois" (dispara benchmark opcional)
```

## 4. Game Boost (Event-Driven)

```
[GameDetector] (ETW foreground/fullscreen) ── jogo iniciou ──▶
   carrega GameProfile → snapshot estado → aplica (power/priority/suspend) → emite "gameboost:on"
[GameDetector] ── jogo encerrou / watchdog ──▶ restore(snapshot) → emite "gameboost:off"
```

## 5. Benchmark + Comparação Antes/Depois

```
run_benchmark(suite) → BenchmarkResult (bruto) → storage
comparação = result_after vs result_before (mesma versão de suite) → delta % por categoria
```

## 6. Relatórios

```
generate_report(scope) → coleta (Diagnosis + Score + Benchmarks + Otimizações + Twin)
   → render template HTML → (PDF engine) → salva + abre
```

## 7. Contratos & tipagem ponta-a-ponta

`tk-contracts` (Rust) `#[derive(Serialize, TS)]` → gera `*.d.ts` → `apps/desktop/src/shared/ipc/types.ts`. Comandos e payloads são **um único source of truth**, eliminando drift entre Rust e TS.
