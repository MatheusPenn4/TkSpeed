# 01 · Product Requirements Document (PRD) — TkSpeed

> Documento vivo. Versão 1.0 · Owner: Product. Audiência: produto, engenharia, design, investidores.

## 1. Visão do produto

> **TkSpeed é a plataforma inteligente de performance para Windows** — um único produto que diagnostica, monitora, otimiza (de forma 100% reversível), mede ganhos reais e aprende o comportamento da máquina ao longo do tempo.

Não é um limpador. É a camada de inteligência de performance entre o usuário e o Windows.

## 2. Problema que resolve

O ecossistema atual obriga o usuário a:
- usar **5+ ferramentas** desconexas (monitor, otimizador, benchmark, game booster, gestor de drivers);
- confiar em otimizadores com **reputação duvidosa** (falsos positivos, mudanças irreversíveis, adware);
- otimizar **às cegas**, sem medir se houve ganho real;
- correr **risco** de quebrar o sistema sem caminho de volta.

**Dores centrais:** fragmentação, falta de confiança, ausência de mensuração, risco irreversível, opacidade técnica.

## 3. Público-alvo

| Segmento | Tamanho relativo | Disposição a pagar |
|---|---|---|
| Gamers entusiastas | Grande | Média-alta |
| Criadores de conteúdo / profissionais | Médio | Alta |
| Técnicos de TI / assistências | Médio | Alta (B2B) |
| Power users | Médio | Média |
| Empresas (frotas de PCs) | Nicho de alto valor | Muito alta |

## 4. Personas

**P1 — Rafael, 24, Gamer competitivo.** Quer FPS estável e 1% low alto. Frustração: stutter inexplicável. Valoriza Game Boost e comparação antes/depois. Paga por Pro.

**P2 — Marina, 31, Editora de vídeo freelancer.** Quer estabilidade térmica em renders longos e diagnóstico confiável. Frustração: throttling. Valoriza monitoramento e relatórios. Paga por Pro.

**P3 — Carlos, 38, Dono de assistência técnica.** Atende 20+ máquinas/semana. Quer laudo profissional para entregar ao cliente e provar valor do serviço. Valoriza relatórios white-label e histórico. Paga Studio.

**P4 — Helena, 45, Gerente de TI (PME, 120 PCs).** Quer padronizar performance e reduzir chamados. Valoriza fleet, política central e auditoria. Paga Enterprise.

**P5 — Diego, 19, Usuário casual com PC lento.** Quer "deixar mais rápido" sem medo de quebrar. Entra no Free; convertido por valor demonstrado.

## 5. Diferenciais competitivos

1. **Reversibilidade total** — toda ação tem snapshot + rollback verificado.
2. **Mensuração real** — benchmark antes/depois prova o ganho (anti-placebo).
3. **Inteligência explicável** — gargalos com problema→impacto→solução, não listas cruas.
4. **Digital Twin** — histórico que detecta regressões ("driver X derrubou 8% do FPS").
5. **Honestidade** — sem falsos positivos, sem adware, telemetria local.
6. **Premium + leve** — UX de classe Tesla/Apple com footprint mínimo (Rust/Tauri).
7. **Plataforma única** — substitui 5 ferramentas.

## 6. Casos de uso (resumo)

Diagnóstico completo · Otimização segura mensurada · Rollback granular · Game Boost automático · Detecção de regressão (Twin) · Relatório profissional · Monitoramento contínuo · Limpeza segura com quarentena. *(Detalhe em [../12-CASOS-DE-USO.md](../12-CASOS-DE-USO.md).)*

## 7. Jornada do usuário

```
Descoberta → Instalação → Primeiro Scan (WOW: score + gargalos claros)
   → Otimização guiada (preview → aplicar → medir ganho)
   → Confiança (rollback disponível, nada quebrou)
   → Hábito (monitoramento + Game Boost no dia a dia)
   → Conversão (recurso bloqueado / histórico > 7d / relatório)
   → Retenção (Digital Twin vira memória da máquina)
   → Advocacy (compartilha relatório/score)
```

**Aha-moment:** ver o TkSpeed Score subir **com ganho comprovado em benchmark** e saber que pode reverter.

## 8. Requisitos funcionais (RF)

| ID | Requisito | Prioridade |
|---|---|---|
| RF-01 | Coletar telemetria em tempo real (CPU/GPU/RAM/storage/rede/temp/FPS) | Must |
| RF-02 | Detectar gargalos e explicá-los (problema/impacto/solução) | Must |
| RF-03 | Calcular TkSpeed Score 0–1000 com breakdown | Must |
| RF-04 | Aplicar otimizações com snapshot + log + rollback | Must |
| RF-05 | Reverter total e granularmente | Must |
| RF-06 | Benchmark e comparação antes/depois | Must |
| RF-07 | Game Boost com perfis por jogo | Should |
| RF-08 | Digital Twin (histórico + regressões) | Should |
| RF-09 | Relatórios PDF/HTML (white-label no Studio) | Should |
| RF-10 | Licenciamento Free/Pro/Studio/Enterprise | Must (1.0) |
| RF-11 | Auto-update assinado | Must |
| RF-12 | Plugins (SDK + sandbox) | Could (Fase 5) |
| RF-13 | TkAI assistente | Could (Fase 4) |

## 9. Requisitos não funcionais (RNF)

| ID | Categoria | Alvo |
|---|---|---|
| RNF-01 | Footprint | < 1% CPU idle, < 150 MB RAM |
| RNF-02 | Cold start | < 1.5 s até dashboard interativo |
| RNF-03 | Segurança | menor privilégio; binários e updater assinados |
| RNF-04 | Confiabilidade | rollback success rate ≥ 99.9%; crash rate < 0.5% |
| RNF-05 | Privacidade | telemetria local; envio só opt-in (LGPD/GDPR) |
| RNF-06 | Compatibilidade | Windows 10 22H2+ e Windows 11 (x64, ARM64 futuro) |
| RNF-07 | i18n | PT-BR, EN, ES no 1.0 |
| RNF-08 | Acessibilidade | AA de contraste, navegação por teclado |
| RNF-09 | Observabilidade | logs estruturados locais; crash reports opt-in |

## 10. Critérios de sucesso

- Usuário completa scan→otimização→medição na 1ª sessão.
- Ganho mediano de score pós-otimização ≥ +5%.
- Taxa de rollback por arrependimento < 5% (sinal de confiança nas mudanças).
- Crash rate < 0.5%.

## 11. Métricas de produto & KPIs

**North Star Metric:** nº de otimizações aplicadas **e mantidas** por usuário ativo/mês.

| Categoria | KPI | Alvo inicial |
|---|---|---|
| Ativação | % que faz 1º scan em <24h | > 70% |
| Ativação | % que aplica ≥1 otimização | > 40% |
| Engajamento | DAU/MAU | > 25% |
| Retenção | D30 | > 35% |
| Monetização | conversão Free→Pro | 3–5% |
| Monetização | churn mensal pago | < 4% |
| Monetização | LTV/CAC | > 3 |
| Satisfação | NPS | ≥ 60 |
| Qualidade | crash-free sessions | > 99.5% |

## 12. Roadmap de evolução

Resumo: **MVP → Beta → 1.0 Comercial → IA (TkAI) → Ecossistema (plugins/fleet)**. Detalhe em [../07-ROADMAP.md](../07-ROADMAP.md) e no [Plano de Lançamento](07-PLANO-LANCAMENTO.md).

## 13. Fora de escopo (não-objetivos)

- Não é antivírus/antimalware.
- Não faz overclock automático agressivo (apenas perfis seguros; OC avançado é opt-in com avisos).
- Não altera o SO de forma irreversível.
- Não é multiplataforma no curto prazo (Windows-first; macOS/Linux só se o mercado justificar).
