# 03 · Estratégia de Benchmark, Baseline & Comparação

> Onde mora o "provar ganho com evidência". A comparação **nunca** afirma ganho que
> esteja dentro do ruído de execução-a-execução.

## 1. Tipos de sessão de benchmark

| Tipo | Mede | Como |
|---|---|---|
| `game_capture` | FPS, 1%/0.1% low, frametime + util/temp/clock | captura ETW durante gameplay real (ou cena repetível) |
| `synthetic_cpu` | tempo/throughput CPU (single/multi) | carga determinística (compressão/cripto/hash) |
| `synthetic_ram` | bandwidth / latência | varredura de memória controlada |
| `synthetic_io` | IOPS / latência seq+rand | leitura/escrita em arquivo temporário (sandbox) |

Sintéticos dão **repetibilidade alta** (controlamos a carga); `game_capture` dá **realismo** (mede o que o usuário sente). O Lab suporta ambos.

## 2. Protocolo de medição (rigor)
1. **Condições controladas e registradas**: resolução, preset, plano de energia, build do SO → salvas em `conditions_json`. Comparação exige **mesmas condições**.
2. **Warmup**: descartar os primeiros N segundos (cache/shaders/boost térmico estabilizando).
3. **Múltiplas execuções**: `runs ≥ 3` (default 5). Cada run gera agregados; guardamos **média e desvio-padrão** entre runs (`benchmark_metrics.stddev`).
4. **Janela fixa**: mesma duração por run.
5. **Outliers**: marcados (ex.: alt-tab, throttle súbito) e sinalizados, não silenciosamente descartados.
6. **Comparabilidade**: só compara sessões de **mesma `suite_version`** e mesmas condições.

## 3. Baseline — fotografia completa da máquina
`capture_baseline(label)` grava um retrato imutável:
- **Hardware** (HardwareInfo: CPU/cores/RAM/discos).
- **Drivers** (versão/data de GPU, chipset, rede, storage).
- **TkSpeed Score** + breakdown do momento.
- **Contexto** (build do Windows, plano de energia, resolução, itens de inicialização — resumo).
- (Opcional) um benchmark de referência associado.

Baselines são a âncora do "antes" e alimentam o **Digital Twin** (regressões ao longo do tempo).

## 4. Motor de Comparação (antes vs depois)

Para cada métrica `m`:
```
delta_abs = after.mean - before.mean
delta_pct = delta_abs / before.mean * 100
margem    = k * sqrt(before.sd² / before.runs + after.sd² / after.runs)   // erro padrão combinado, k≈2 (~95%)
```
Classificação:
- **Ganho** se `|delta| > margem` **e** na direção boa (mais FPS/1%low, menos frametime/temp).
- **Perda** se `|delta| > margem` na direção ruim.
- **Sem alteração** se `|delta| ≤ margem` → rotulado *"dentro da margem de erro (±X%)"*.

> Regra de ouro: **se o delta não supera a margem de erro, o TkSpeed NÃO afirma ganho.**
> Toda alegação na UI carrega a margem (ex.: "+7,3% no 1% low, ±1,1%, 5 runs").

Métricas-chave priorizadas no veredito: **1% low** e **0.1% low** (o que o usuário sente como travamento) > FPS médio > frametime P99 > temperaturas/clock.

## 5. Saída
`comparisons.verdict_json` = lista de `{ metric, before, after, delta_pct, margin_pct, classification }`.
A UI mostra a tabela antes/depois com setas verde/vermelho/cinza e a margem por linha.
