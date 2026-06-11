# 06 · Backlog Executável & Plano por Slices

Padrão EPIC → STORY (P0–P3, esforço XS–XL, risco 🟢🟡🔴). **Slices incrementais e validáveis**,
cada uma fechando o loop para um conjunto pequeno. QA obrigatório por slice (fim do doc).

## Habilitador técnico (pré-requisito da OE-1)
**Generalizar o `ProtectionService`** (hoje preso ao piloto MenuShowDelay) para capturar/restaurar
um conjunto de `ReversibleAction` (registry HKCU, quarentena de arquivo, plano de energia, serviço).
Isso é o que permite qualquer otimização ser reversível com a infra já validada.

---

## TK-EO01 · Núcleo do Engine + 1 SAFE (prova o loop sem benchmark)
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| EO-S0101 | Generalizar `ProtectionService` p/ `ReversibleAction[]` (registry/quarentena/power/serviço) | P0 | L | 🔴 |
| EO-S0102 | Trait `Optimization` + `OptimizationMeta` + `catalog()` | P0 | M | 🟢 |
| EO-S0103 | `TkOptimizationExecutor` (preview→capture→snapshot→apply→verify, rollback em falha) | P0 | L | 🔴 |
| EO-S0104 | SAFE piloto `cleanup.temp_files` (quarentena, `Validation=SpaceFreed`) | P0 | M | 🟡 |
| EO-S0105 | Migração `0004` + `OptimizationRun`/`EvidenceRecord` + `OptRepo` | P0 | M | 🟢 |
| EO-S0106 | Comandos `opt_catalog`/`opt_preview`/`opt_run`/`opt_history`/`opt_rollback` + `AppContext.optimize()` | P0 | M | 🟢 |
| EO-S0107 | UI "Centro de Otimizações" (lista, risco, impacto, aplicar, histórico, reverter) | P0 | L | 🟢 |
> Resultado: aplicar/reverter uma otimização SAFE com snapshot+evidência+auditoria, ponta a ponta.

## TK-EO02 · Validator + Evidência por benchmark (prova "ganho com evidência")
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| EO-S0201 | `TkOptimizationValidator` (bench antes/depois via tk-perflab + Confidence Engine) | P0 | L | 🟡 |
| EO-S0202 | `TkOptimizationEvidence` (score+bench antes/depois, ganho real, confiança → persist) | P0 | M | 🟢 |
| EO-S0203 | MODERATE piloto `energy.power_plan_high` (full pipeline c/ `cpu-1.0.0`) | P0 | M | 🔴 |
| EO-S0204 | Decisão Keep/Revert/Inconclusive + reversão automática | P0 | M | 🔴 |
| EO-S0205 | UI: impacto **medido** (Δ ± margem, confiança, decisão) por run | P1 | M | 🟢 |
> Resultado: uma otimização MODERATE é aplicada, medida e **mantida só se comprovou ganho**.

## TK-EO03 · Advisor
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| EO-S0301 | `advise(bottleneck, history)` → recomendações (motivo/impacto/risco/evidência) | P1 | M | 🟡 |
| EO-S0302 | Honestidade: não recomendar onde o SO não resolve (GPU/Thermal) | P1 | S | 🟢 |
| EO-S0303 | UI: recomendações no Centro de Otimizações ligadas ao detector | P1 | M | 🟢 |

## TK-EO04 · Mais SAFE (limpeza + recomendações)
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| EO-S0401 | `cleanup.wu_cache`, `cleanup.old_logs` (quarentena) | P1 | M | 🟡 |
| EO-S0402 | `startup.analyze` (read-only) + `storage.recommend` (read-only) | P1 | M | 🟢 |

## TK-EO05 · MODERATE (lote)
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| EO-S0501 | `energy.game_mode`, `energy.usb_selective_suspend_off` (HKCU/power) | P2 | M | 🟡 |
| EO-S0502 | `services.optional_off` com **whitelist curada** + blacklist de segurança | P2 | L | 🔴 |

## TK-EO06 · Perfis
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| EO-S0601 | `TkOptimizationProfiles` (7 perfis built-in, composição rastreável) | P2 | M | 🟢 |
| EO-S0602 | `run_profile` (snapshot único + pipeline por item + relatório agregado + reverter em bloco) | P2 | L | 🟡 |
| EO-S0603 | UI de perfis (mostra itens, risco, ganhos comprovados vs inconclusivos) | P2 | M | 🟢 |

## TK-EO07 · ADVANCED + EXPERIMENTAL
| ID | Story | P | Esf | Risco |
|---|---|:--:|:--:|:--:|
| EO-S0701 | Opt-in + confirmação de risco + integração com Ponto de Restauração do Windows | P3 | M | 🔴 |
| EO-S0702 | `registry.documented_tweaks`, `process.affinity`, `scheduler.tweaks`, `memory.advanced` | P3 | L | 🔴 |
| EO-S0703 | Canal EXPERIMENTAL (oculto, flag de usuário avançado) | P3 | S | 🟡 |

## Ordem recomendada
```
[habilitador EO-S0101] → OE-1 (loop SAFE) → OE-2 (validação por benchmark) → OE-3 (advisor)
   → OE-4 (mais SAFE) → OE-5 (MODERATE) → OE-6 (perfis) → OE-7 (advanced/experimental)
```
**1ª slice executável recomendada: OE-1** — prova o loop fechado (snapshot→aplicar→evidência→reverter)
com uma otimização **SAFE** (limpeza com quarentena), sem nem precisar de benchmark. Depois OE-2 pluga
o benchmark/Confidence e o produto passa a **provar ganho com evidência**.

## QA OBRIGATÓRIO (gate de cada slice)
Nenhuma otimização/slice é "concluída" sem:
- ✅ build verde (`cargo build` + frontend `tsc`/`vite`)
- ✅ testes verdes (unit do domínio: critérios de decisão, evidência, catálogo)
- ✅ **rollback validado** (restaura estado original, verificado, em VM/host)
- ✅ **benchmark validado** (antes/depois roda e alimenta o Confidence Engine)
- ✅ **comparação validada** (Keep/Revert/Inconclusive corretos por testes sintéticos)
- ✅ sem alegação de ganho sem evidência; impacto sempre com margem + confiança

## Definição de Pronto (Engine)
Uma otimização só é "Mantida" quando: snapshot criado + benchmark antes/depois + Confidence Engine
confirmou ganho acima da margem dinâmica + auditoria registrada. Caso contrário: **revertida** e
marcada **inconclusiva** — com o motivo visível ao usuário.
