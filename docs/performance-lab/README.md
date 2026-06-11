# TkPerformanceLab — Fundação de Medição

> **Princípio inegociável:** toda otimização futura do TkSpeed **deve ser medida**.
> O produto **nunca** afirma ganho sem evidência reproduzível e estatisticamente significativa.

Este módulo é construído **antes** de qualquer otimização real. Ele dá ao TkSpeed a
capacidade de **medir, registrar, comparar e explicar** o desempenho — transformando
"deixamos seu PC mais rápido" (alegação) em "medimos +7,3% de 1% low, com margem de
erro de ±1,1%, em 5 execuções controladas" (prova).

## Índice

| Doc | Conteúdo | Entregável |
|---|---|---|
| [01 · Arquitetura](01-ARQUITETURA.md) | Componentes, crate `tk-perflab`, modelo de dados, **fluxo** | 1, 3 |
| [02 · Métricas & Coleta](02-METRICAS-E-COLETA.md) | Catálogo de métricas, como coletar, custo e impacto | 4 (item), 5 |
| [03 · Benchmark, Baseline & Comparação](03-BENCHMARK-COMPARACAO.md) | Estratégia de benchmark, baseline, motor de comparação + significância | 4 |
| [04 · Detectores de Gargalo](04-DETECTORES.md) | CPU/GPU/RAM/Storage/Thermal bound | 6 (item) |
| [05 · Wireframe](05-WIREFRAME.md) | Tela Performance Lab | 7 (item) |
| [06 · Backlog](06-BACKLOG.md) | Epics → stories para construir a capacidade | 2 |
| [07 · Critérios de Validação](07-VALIDACAO.md) | Como confiamos nos números | 6 |

## Componentes (visão rápida)

```
TkPerformanceLab            facade / casos de uso (orquestra tudo)
 ├── TkMetricsCollector     coleta multi-fonte (sysinfo, PDH-GPU, ETW-FPS, LHM)
 ├── TkBenchmarkEngine      sessão de captura + agregação (FPS, lows, frametime, %CPU/GPU…)
 ├── Baseline               fotografia completa da máquina (HW+drivers+score+telemetria+bench)
 ├── TkComparisonEngine     antes vs depois com margem de erro / significância
 └── Bottleneck Detectors   CPU/GPU/RAM/Storage/Thermal bound
```

## Limites desta fase (o que NÃO entra agora)
- ❌ Nenhuma otimização real, nenhum Game Boost, nenhuma alteração no Windows.
- ✅ Apenas a **capacidade de medir**. Otimizações virão depois — e só serão lançadas se o Lab provar o ganho.
