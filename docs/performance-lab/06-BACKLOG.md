# 06 · Backlog — Construir a capacidade de medir

Mesmo padrão do backlog do MVP (EPIC → STORY, prioridade P0–P3, esforço XS–XL, risco 🟢🟡🔴).
**Escopo: só medição.** Nenhuma otimização, nenhum Game Boost, nenhuma alteração no SO.
Ordem = de baixo para cima (coleta → engine → comparação → UI).

## PL-E01 · Camada de coleta (`tk-perflab` + `tk-platform-win`)
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| PL-S0101 | Trait `MetricsCollector` + `MetricFrame`/`MetricPoint` + agregador de sessão | P0 | M | 🟢 |
| PL-S0102 | `SysinfoCollector` (reusa lógica atual: CPU/RAM/disco) | P0 | S | 🟢 |
| PL-S0103 | `PdhGpuCollector` — GPU usage + VRAM via contadores PDH (sem elevação) | P0 | L | 🟡 |
| PL-S0104 | `PdhSystemCollector` — clock CPU, paging (Pages/sec), disco (latência/queue) | P1 | M | 🟡 |
| PL-S0105 | `EtwFpsCollector` — present events → frametime/FPS (sessão ETW, **elevação**) | P1 | XL | 🔴 |
| PL-S0106 | `LhmCollector` — temp/clock/voltagem via LibreHardwareMonitor (opcional, degradável) | P2 | L | 🔴 |
| PL-S0107 | Degradação graciosa + flag de disponibilidade por fonte | P1 | S | 🟢 |

## PL-E02 · Benchmark Engine
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| PL-S0201 | Sessão de captura (start→poll→stop) com ring buffer de frametime | P0 | M | 🟡 |
| PL-S0202 | Agregação: FPS médio, 1% low (P99), 0.1% low (P99.9), frametime P99; %CPU/GPU/RAM/VRAM/temp/clock | P0 | M | 🟢 |
| PL-S0203 | Warmup discard + múltiplos runs + média/desvio entre runs | P0 | M | 🟢 |
| PL-S0204 | Suites sintéticas determinísticas: CPU, RAM, IO (em sandbox, sem tocar dados do usuário) | P1 | L | 🟡 |
| PL-S0205 | Versionamento (`suite_version`) + registro de `conditions` (controle) | P0 | S | 🟢 |

## PL-E03 · Baseline
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| PL-S0301 | Migração `0002_perflab.sql` (perf_baselines/benchmark_sessions/metrics/comparisons) | P0 | S | 🟢 |
| PL-S0302 | `capture_baseline(label)` — HW + drivers + score + contexto | P0 | M | 🟡 |
| PL-S0303 | Auditor de drivers (versão/data GPU/chipset/rede/storage) | P1 | M | 🟡 |
| PL-S0304 | Retenção de baselines/sessões no `housekeeping` existente | P2 | S | 🟢 |

## PL-E04 · Comparison Engine
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| PL-S0401 | `compare(before, after)` com erro padrão combinado + classificação Ganho/Perda/Sem alteração | P0 | M | 🟡 |
| PL-S0402 | Guard de comparabilidade (mesma `suite_version`/condições) | P0 | S | 🟢 |
| PL-S0403 | Persistir veredito + exportar pacote de evidência | P1 | M | 🟢 |
| PL-S0404 | Testes do motor com séries sintéticas (ganho real vs ruído) | P0 | M | 🟢 |

## PL-E05 · Detectores de gargalo
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| PL-S0501 | GPU/CPU bound (correlação %GPU/%CPU + frametime) | P1 | M | 🟡 |
| PL-S0502 | RAM/Storage bound (paging/latência correlacionados a spikes) | P2 | M | 🟡 |
| PL-S0503 | Thermal bound (queda de clock + temp) — requer LHM/PDH | P2 | M | 🔴 |

## PL-E06 · UI Performance Lab + bridge
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| PL-S0601 | Comandos: `perf_capture_baseline`, `perf_run_benchmark`, `perf_compare`, `perf_list_baselines` + tipos em `tk-contracts` | P0 | M | 🟢 |
| PL-S0602 | Evento de streaming `perf:frame` (frametime ao vivo) + `perf:progress` | P1 | M | 🟢 |
| PL-S0603 | Tela Performance Lab: config de sessão + frametime ao vivo + veredito | P0 | L | 🟢 |
| PL-S0604 | Tela de comparação antes/depois (tabela com Δ ± margem) | P0 | M | 🟢 |
| PL-S0605 | Baselines/histórico + ligação ao Digital Twin | P2 | M | 🟢 |

## Ordem ideal / caminho crítico
```
PL-S0101 (collector trait) → PL-S0103 (GPU PDH) ┐
PL-S0102 (sysinfo) ─────────────────────────────┼→ PL-S0201/0202/0203 (engine)
PL-S0301 (migração) ─────────────────────────────┘        │
                                                           ▼
                                          PL-S0401 (comparison) → PL-S0601/0603/0604 (UI)
EtwFpsCollector (PL-S0105) é paralelo e de risco alto (ETW+elevação) — não bloqueia
o caminho sintético; FPS de jogo entra quando estável.
```

> Sugestão de 1ª slice executável (quando aprovado): **PL-S0101 + PL-S0102 + PL-S0301 + PL-S0202 + PL-S0204(CPU)** — já permite um benchmark sintético de CPU/RAM com baseline e agregação, sem ETW/elevação. FPS de jogo (ETW) vira uma slice dedicada depois.
