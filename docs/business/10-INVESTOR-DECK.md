# 10 · TkSpeed — Documento de Investimento / Sócio Técnico

> Material executivo equivalente a um pitch para investidores e potenciais sócios técnicos. Sintetiza produto, tecnologia, mercado, modelo e estratégia. Aprofundamento em cada artefato de `docs/` e `docs/business/`.

---

## 1. Resumo executivo

**TkSpeed é a Plataforma Inteligente de Performance para Windows.** Num único produto premium, ela **diagnostica, monitora, otimiza (de forma 100% reversível), mede ganhos reais e aprende o comportamento da máquina** (Digital Twin). Substitui 5+ ferramentas fragmentadas e ataca a maior fraqueza da categoria: **falta de confiança e de mensuração**.

- **Categoria:** software de performance/utilitário desktop (freemium + assinatura).
- **Stack:** Rust + Tauri + React + SQLite — premium e leve, com arquitetura de nível SaaS.
- **Moat:** reversibilidade total + Digital Twin histórico + IA local-first + plataforma de plugins.
- **Estágio:** arquitetura e fundação de produto completas; pronto para construir MVP.

## 2. O problema

Otimizar um PC Windows hoje é **fragmentado, arriscado e cego**:
- 5+ apps desconexos (monitor, otimizador, benchmark, booster, drivers);
- otimizadores com **reputação tóxica** (falsos positivos, adware, mudanças irreversíveis);
- **nenhuma medição** real do ganho ("efeito placebo" da otimização);
- **medo** justificado de quebrar o sistema sem volta.

## 3. A solução

Um ciclo único, confiável e mensurável:

```
MONITORAR → DIAGNOSTICAR (gargalo explicado) → OTIMIZAR (snapshot+rollback)
→ MEDIR (benchmark antes/depois) → LEMBRAR (Digital Twin) → RECOMENDAR (TkAI)
```

**O que nos torna diferentes:** cada mudança é reversível e auditável; cada ganho é provado; cada regressão é detectada ("o driver X derrubou 8% do seu FPS"). Honestidade vira vantagem competitiva.

## 4. Por que agora

- PC gaming e criação de conteúdo em alta estrutural; usuários cada vez mais exigentes com performance.
- Desconfiança acumulada com otimizadores legados (espaço para um player **honesto**).
- Maturidade de Rust/Tauri permite produto **premium e leve** que antes era caro de construir.
- IA local barata o suficiente para assistente de performance **privado**.

## 5. Produto

Dez telas (Dashboard, Análise, Monitoramento, Game Boost, Benchmark, Histórico/Twin, Relatórios, Diagnóstico, Rollback, Settings). Design system premium "Aurora" (futurista, glassmorphism, dados em tempo real). Detalhe: [PRD](01-PRD.md), [Design System](../06-DESIGN-SYSTEM.md), [Wireframes](../16-WIREFRAMES.md).

**TkSpeed Score (0–1000):** métrica proprietária, explicável e comparável no tempo — o "número" que vira hábito e prova de valor.

## 6. Tecnologia & defensabilidade (moat)

| Camada de moat | Por quê é difícil de copiar |
|---|---|
| **Reversibilidade (Saga + Rollback)** | Exige arquitetura desde o núcleo; legados não conseguem retrofitar |
| **Digital Twin** | Dados históricos por máquina criam lock-in — quanto mais tempo, mais valioso e insubstituível |
| **Arquitetura Clean/DDD em Rust** | Qualidade/segurança/leveza que OEM bloatware não alcança |
| **IA local-first sobre o Twin** | Privacidade + custo ~zero + contexto único da máquina |
| **Plataforma de plugins (WASM)** | Efeito de rede de ecossistema/marketplace |

Decisões fundamentadas em [ADRs](02-ADR.md). Arquitetura em [docs/01](../01-ARQUITETURA.md).

## 7. Mercado

- **TAM:** ~1,4 bi PCs Windows. **SAM:** ~150–250 mi (gamers/criadores/técnicos/PMEs). **SOM (3 anos):** ~300 mil–1 mi instalações.
- Detalhe e premissas: [Mercado & Receita](09-MERCADO-RECEITA.md).

## 8. Modelo de negócio

Freemium + assinatura, **offline-first**, 4 planos:

| Plano | Alvo | Preço alvo |
|---|---|---|
| Free | adoção/funil | R$ 0 |
| Pro | gamers/power users | ~R$ 199/ano |
| Studio | técnicos/assistências (white-label) | ~R$ 699/ano |
| Enterprise | PMEs/frotas | sob consulta (por seat) |

Margem bruta > 85% (custo marginal ~0; cloud só opt-in). Detalhe: [Licenciamento](03-LICENCIAMENTO.md).

### Projeção (cenário base)
| Base instalada | ARR |
|---|--:|
| 1.000 | ~R$ 10,5 mil |
| 10.000 | ~R$ 105 mil |
| 100.000 | ~R$ 1,04 mi |

Upside: Enterprise + marketplace de plugins (não modelados no base).

## 9. Go-to-market

1. **Wedge gamer** (Game Boost + FPS mensurável) — viral, engajado, alta DAP.
2. **Afiliados/criadores** de tech & gaming; benchmarks "antes/depois" compartilháveis.
3. **SEO de dores** ("por que meu FPS caiu", "PC esquentando") — captura intenção dos concorrentes.
4. **B2B**: assistências (Studio white-label) → PMEs (Enterprise).
5. **PLG**: produto Free excelente como principal canal de aquisição. Detalhe: [Diferenciação](04-DIFERENCIACAO.md).

## 10. Concorrência

Ninguém entrega o **ciclo completo, reversível, mensurável, inteligente e agnóstico**. CCleaner/ASC (limpeza tóxica), Razer Cortex (boost raso), Process Lasso (só CPU), HWInfo (só monitora), OEM (travado em marca). Matriz completa em [04](04-DIFERENCIACAO.md). **Oceano azul** confirmado.

## 11. Roadmap & lançamento

`MVP → Alpha → Beta → v1.0 (comercial) → v2.0 (IA + ecossistema)`. Cada estágio prova uma hipótese; segurança (auditoria externa) é gate do 1.0. Detalhe: [Plano de Lançamento](07-PLANO-LANCAMENTO.md) e [Roadmap](../07-ROADMAP.md).

## 12. Riscos & mitigação

Top riscos: quebrar o SO (mitigado pela arquitetura de rollback — o pilar do produto), posicionamento confundido com "otimizador comum", abuso do componente elevado, footprint da ferramenta, conversão. Matriz completa + contingências (kill switch de tweak, rollback em massa) em [Matriz de Riscos](08-MATRIZ-RISCOS.md).

## 13. Time & necessidades

- **Fundadores:** visão de produto + arquitetura (já materializada neste repositório).
- **Sócio técnico ideal:** Principal Engineer Rust/Windows (núcleo, adapters de sensores, componente elevado).
- **Primeiras contratações pós-investimento:** 1–2 eng. Rust sêniores, 1 eng. frontend/React, 1 designer de produto, 1 growth/PLG.
- **Bus factor** mitigado por documentação completa (este repositório é o "blueprint" da empresa).

## 14. O pedido (uso de recursos — ilustrativo)

Captação **seed** para levar do MVP ao 1.0 comercial (~12–18 meses):

| Alocação | % |
|---|--:|
| Engenharia (núcleo Rust + frontend) | ~55% |
| Design & produto | ~12% |
| Segurança & auditoria externa | ~8% |
| Go-to-market (afiliados, conteúdo, ASO) | ~18% |
| Infra & licenciamento/faturamento | ~7% |

**Marcos de desbloqueio:** Beta com D30 > 30% e NPS ≥ 50 → liberar push comercial; 1.0 com conversão ≥ 3% e auditoria limpa → escalar GTM.

## 15. Por que vai dar certo

1. **Problema real e doloroso**, com concorrentes que perderam a confiança do usuário.
2. **Diferenciação defensável** (reversibilidade + Twin + IA local + plugins) — não é feature, é arquitetura.
3. **Economia de software** com margem alta e custo marginal quase zero.
4. **Fundação técnica já pronta** — arquitetura, contratos, scaffolding e blueprint completos reduzem risco de execução.
5. **Múltiplos motores de upside** (Pro → Studio → Enterprise → marketplace) sem inflar custo do núcleo.

---

### Índice de aprofundamento
PRD [01](01-PRD.md) · ADR [02](02-ADR.md) · Licenciamento [03](03-LICENCIAMENTO.md) · Diferenciação [04](04-DIFERENCIACAO.md) · Plugins [05](05-PLUGINS.md) · TkAI [06](06-TKAI-IA.md) · Lançamento [07](07-PLANO-LANCAMENTO.md) · Riscos [08](08-MATRIZ-RISCOS.md) · Mercado/Receita [09](09-MERCADO-RECEITA.md) · Arquitetura técnica [`../`](../).
