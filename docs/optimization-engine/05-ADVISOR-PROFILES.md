# 05 · Advisor (inteligência de decisão) & Perfis

## 1. Advisor — do gargalo à ação
Entrada: gargalo detectado por `tk-perflab::detect` / `tk-analyzer` + telemetria + histórico de evidências.
Saída: recomendações com **motivo · impacto esperado · risco · evidência prévia**. **Não aplica nada.**

| Gargalo | Recomenda (ids do catálogo) | Motivo (exemplo) |
|---|---|---|
| **CPU Bound** | `services.optional_off`, `process.affinity` (adv), `energy.power_plan_high` | CPU saturada e GPU ociosa → reduzir trabalho de fundo / priorizar |
| **GPU Bound** | (poucas ações de SO ajudam) → orienta drivers/qualidade; evita prometer | GPU é o limite → ganho de SO tende a ser marginal (honesto) |
| **RAM Bound** | `cleanup.*`, `startup.analyze`, `memory.advanced` (adv) | pressão de memória → liberar/reduzir residentes |
| **Storage Bound** | `cleanup.*`, `storage.recommend` | pouco espaço/HDD → liberar espaço; recomendar SSD |
| **Thermal Bound** | orienta (limpeza física/ventilação); `energy` perfis menos agressivos | throttling térmico → SO não resolve calor (honesto) |
| **Balanceado/Ocioso** | nenhuma ação automática | "sem gargalo dominante — execute sob carga real" |

Princípio: **quando o SO não resolve o gargalo (GPU/Thermal), o Advisor diz isso** em vez de
empurrar tweaks inúteis. Cada recomendação carrega a **evidência prévia** daquela otimização
naquela máquina (ex.: "rendeu +6% no 1% low, ±1.4%, 5 runs") quando existir histórico.

```rust
fn advise(bottleneck: BottleneckKind, history: &EvidenceHistory) -> Vec<AdvisorRecommendation>
```

## 2. Perfis de Otimização (bundles rastreáveis)
Cada perfil é uma lista de ids do catálogo (composição transparente). Aplicar = pipeline por item,
com snapshot único de perfil (reversível em bloco).

| Perfil | Composição (ids) | Foco |
|---|---|---|
| **Competitive Gaming** | `energy.power_plan_high`, `energy.game_mode`, `energy.usb_selective_suspend_off`, `services.optional_off` | latência mínima / 1% low |
| **AAA Gaming** | `energy.power_plan_high`, `energy.game_mode`, `cleanup.temp_files` | FPS médio + folga |
| **Streaming** | `services.optional_off` (conservador), `energy.power_plan_high` | estabilidade de encode + jogo |
| **Content Creator** | `energy.power_plan_high`, `cleanup.*` | throughput sustentado (CPU/IO) |
| **Balanced** | `cleanup.temp_files`, `startup.analyze` (recomenda), `energy.power_plan_high` | ganho seguro, baixo risco |
| **Maximum Performance** | tudo de Competitive + ADVANCED opt-in | desempenho máx. (exige confirmação) |
| **Power Saving** | plano de energia equilibrado/econômico, USB suspend on | bateria/temperatura |

Regras dos perfis:
- **Rastreável:** a UI sempre mostra exatamente quais otimizações o perfil contém e o risco de cada.
- **Mesma exigência de evidência:** cada item do perfil passa pelo pipeline e gera evidência própria;
  o perfil mostra um **relatório agregado** (ganhos comprovados vs inconclusivos vs revertidos).
- **Reversível em bloco:** "Reverter perfil" restaura o snapshot do perfil inteiro.
- Perfis que incluem ADVANCED exigem opt-in + confirmação de risco.

## 3. Honestidade do Advisor (anti-snake-oil)
- Nunca recomenda algo cujo gargalo o SO não endereça (ex.: tweak de registro para GPU-bound).
- Nunca promete número sem evidência; usa "impacto esperado" qualitativo até medir, e troca por
  **ganho medido ± margem** após o pipeline.
- Recomendações com histórico negativo na máquina são **despriorizadas/ocultadas** ("já não ajudou aqui").
