# 07 · Critérios de Validação — "podemos confiar no número?"

O TkPerformanceLab só cumpre seu propósito se os números forem **confiáveis**. Estes são os
critérios que tornam uma medição (e, portanto, uma alegação de ganho) válida.

## 1. Critérios de confiabilidade da medição

| # | Critério | Limite / regra |
|---|---|---|
| V1 | **Baixo observer effect** | Overhead total da coleta durante captura **< 2% CPU**; capturar com e sem o Lab e confirmar que o FPS médio não muda além da margem. |
| V2 | **Repetibilidade** | Em condições fixas, o coeficiente de variação entre runs do **FPS médio** deve ser **< 3%** (sintéticos < 1%). Acima disso → medição "instável", não comparável. |
| V3 | **Calibração externa** | FPS/frametime do `EtwFpsCollector` devem bater (±2%) com uma ferramenta de referência (PresentMon/CapFrameX) na mesma cena. |
| V4 | **Determinismo dos sintéticos** | Suites CPU/RAM/IO repetem o mesmo trabalho; variância < 1% na mesma máquina/versão. |
| V5 | **Disponibilidade honesta de sensor** | Se uma fonte não existe (sem elevação p/ FPS, sem LHM p/ temp), a métrica é `indisponível` — **nunca** estimada ou inventada. |
| V6 | **Comparabilidade** | Comparar só sessões de mesma `suite_version` **e** mesmas `conditions` (resolução/preset/plano de energia/build). Caso contrário, a comparação é bloqueada. |

## 2. Critérios para AFIRMAR um ganho (gate anti-alegação-falsa)

Uma melhoria só pode ser comunicada como "ganho" se **todos**:
1. `runs ≥ 3` (default 5), com warmup descartado.
2. `|delta| > margem de erro` (erro padrão combinado, ~95%).
3. Mesmas condições controladas e registradas.
4. O **sentido** é benéfico (mais FPS/1%low, menos frametime/temp).
5. (Quando aplicável) o **bound** detectado aliviou de forma coerente com a mudança.

Se qualquer um falhar → rótulo **"Sem alteração / dentro do ruído"**. Nunca "ganho".

## 3. Critérios de aceite do módulo (Definition of Done do Performance Lab)
- [ ] Baseline captura HW + drivers + score + contexto e persiste (migração 0002).
- [ ] Benchmark sintético (CPU/RAM/IO) roda N runs e reporta média ± desvio.
- [ ] `game_capture` (quando elevado) reporta FPS médio, 1% low, 0.1% low, frametime P99.
- [ ] Coleta de %CPU/%GPU/VRAM/RAM funciona **sem elevação** (PDH/sysinfo).
- [ ] ComparisonEngine classifica corretamente ganho/perda/ruído em testes com séries sintéticas (V2/V4 cobertos por teste).
- [ ] Overhead da coleta < 2% (V1) medido e registrado.
- [ ] Calibração V3 documentada (cross-check com ferramenta de referência).
- [ ] UI mostra todo ganho **com margem de erro e nº de runs**; "indisponível" onde falta sensor.

## 4. Testes recomendados
- **Unit:** percentis (1%/0.1% low) sobre frametimes sintéticos conhecidos; ComparisonEngine (ganho real vs ruído vs perda); guard de comparabilidade.
- **Integração:** sessão completa sintética → agregação → persistência → comparação.
- **Calibração (manual/semi):** capturar a mesma cena com o Lab e com PresentMon/CapFrameX e comparar (V3).
- **Overhead (perf):** medir FPS de uma cena com o Lab ligado vs desligado (V1).
- **Robustez:** rodar sem LHM e sem elevação → métricas degradam para "indisponível", nada quebra (V5).

---

**Resumo:** o valor do TkPerformanceLab não é "ter números bonitos" — é **garantir que qualquer
ganho anunciado pelo TkSpeed seja real, medido, reproduzível e dentro de margem de erro
declarada.** Esta é a fundação que torna as otimizações da Fase 2 *defensáveis*.
