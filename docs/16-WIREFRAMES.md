# 16 · Wireframes

Wireframes de baixa fidelidade das telas principais. A versão de alta fidelidade vive no código (`apps/desktop/src/features/*`).

## Dashboard

```
┌─ TkSpeed ───────────────────────────────────────── — ▢ ✕ ┐
│ ◈ Dashboard          Dashboard            [Relatório][Analisar]│
│ ⊹ Análise            Visão geral em tempo real                 │
│ ∿ Monitor      ┌──────────┐ ┌─────────────────┐ ┌───────────┐ │
│ ⚡ GameBoost   │ SCORE    │ │ GARGALOS         │ │ GAME BOOST│ │
│ ▲ Benchmark   │   742    │ │ ● CPU bottleneck │ │   ⚡       │ │
│ ◷ Histórico   │  /1000   │ │ ● 12 startup     │ │  Pronto   │ │
│ ▤ Relatórios  │ Excelente│ │ ● driver antigo  │ │ [Ativar]  │ │
│ ─────────     └──────────┘ └─────────────────┘ └───────────┘ │
│ ✛ Diagnóstico ┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐            │
│ ↺ Rollback    │CPU ││GPU ││RAM ││TEMP││NET ││FPS │            │
│ ⚙ Config      │34% ││21% ││58% ││54° ││12  ││142 │            │
│               │∿∿∿ ││∿∿∿ ││∿∿∿ ││∿∿∿ ││    ││    │            │
└───────────────└────┘└────┘└────┘└────┘└────┘└────┘────────────┘
```

## Central de Diagnóstico

```
┌──────────────────────────────────────────────────────────────┐
│ Central de Diagnóstico                    [Re-analisar]        │
│ Score 742 ▲  •  Última análise há 4 min                        │
├──────────────────────────────────────────────────────────────┤
│ FILTROS: [Todos][CPU][GPU][RAM][Storage][Rede][Térmico]        │
│ ┌────────────────────────────────────────────────────────────┐│
│ │ 🔴 ALTA   Gargalo de CPU                                    ││
│ │   Problema: CPU 95% / GPU 48% em jogos                      ││
│ │   Impacto:  FPS limitado pela CPU                           ││
│ │   Solução:  Game Boost + fechar apps  → [Pré-visualizar]    ││
│ ├────────────────────────────────────────────────────────────┤│
│ │ 🟡 MÉDIA  12 itens de inicialização …        [Resolver]     ││
│ └────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────┘
```

## Monitoramento

```
┌──────────────────────────────────────────────────────────────┐
│ Monitoramento                       [1s ▼] [Exportar CSV]      │
│ ┌─ CPU ───────────────┐ ┌─ GPU ───────────────┐               │
│ │  gráfico tempo real │ │  gráfico tempo real │  por núcleo:  │
│ │  ▁▂▅▇▆▅▃▂  34%      │ │  ▁▁▂▃▅▆  21%        │  [▮▮▮▮▮▮▮▮]   │
│ │  clock 4.6GHz 54°C  │ │  1.8GHz 8GB/16 62°C │               │
│ └─────────────────────┘ └─────────────────────┘               │
│ ┌─ RAM ─┐ ┌─ Disco ─┐ ┌─ Rede ─┐ ┌─ Temperaturas ──────────┐  │
│ │ 9.3GB │ │ NVMe    │ │ ↑2 ↓12 │ │ CPU 54 GPU 62 SSD 41 °C │  │
│ └───────┘ └─────────┘ └────────┘ └─────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Rollback

```
┌──────────────────────────────────────────────────────────────┐
│ Rollback                                                       │
│ LINHA DO TEMPO                          QUARENTENA             │
│ ● 05/06 14:22  Otimização gamer (8 mudanças) [Reverter][Diff] │
│ ● 04/06 09:10  Limpeza segura (3)            [Reverter]       │
│ ● 01/06 20:55  Plano de energia (1)  ✓ ativo                  │
│                                                                │
│ DIFF do snapshot selecionado:                                  │
│  registry HKCU\...\VisualEffects   1 → 2                       │
│  power    active_scheme  Balanceado → Alto Desempenho          │
└──────────────────────────────────────────────────────────────┘
```

## Benchmark / Antes-Depois

```
┌──────────────────────────────────────────────────────────────┐
│ Benchmark            [Rodar suite ▼]   [Comparar antes/depois] │
│ CPU single ███████░ 1820   (+6%)                               │
│ CPU multi  ████████ 14210  (+4%)                               │
│ RAM        ██████░░ 38GB/s (+1%)                               │
│ SSD        ███████░ 3.1GB/s                                    │
│ ► Ganho médio pós-otimização: +4.2%                            │
└──────────────────────────────────────────────────────────────┘
```

## Histórico (Digital Twin)

```
┌──────────────────────────────────────────────────────────────┐
│ Histórico · Digital Twin             [7d][30d][1a]             │
│  Score  ▁▂▃▅▅▆▇  742                                           │
│  marcos: ⬩driver NVIDIA(12/05)  ⬩otimização(05/06)             │
│ ┌ INSIGHT ───────────────────────────────────────────────────┐│
│ │ ⚠ Após o driver NVIDIA 555, FPS médio caiu 8% (142→131).    ││
│ │   [Ver detalhes]   [Reverter driver]                        ││
│ └────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────┘
```
