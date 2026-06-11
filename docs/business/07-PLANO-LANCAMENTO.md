# 07 · Plano de Lançamento

> Filosofia: cada estágio prova **uma** hipótese antes de avançar. Escopo entra por valor/risco, não por "seria legal ter".

## MVP — *Prova o ciclo de valor* (interno/dogfooding)

**Hipótese:** o ciclo monitorar→diagnosticar→otimizar→medir→reverter funciona e encanta.

- **Entra:** TkCore; TkMonitor (CPU/RAM/GPU/storage/temp); TkAnalyzer (gargalo CPU/RAM/storage + startup); TkOptimizer (~10 tweaks `safe`); TkRollback (snapshot/restore); TkBenchmark (CPU/RAM/SSD); TkSpeed Score v1; telas Dashboard, Monitoramento, Diagnóstico, Rollback, Settings.
- **Fica de fora:** Game Boost, Digital Twin, relatórios, licenciamento, IA, plugins, rede/FPS.
- **Prioridade:** confiabilidade do rollback acima de tudo.
- **Saída:** aplicar otimização, medir ganho e reverter 100% em 10+ máquinas internas.

## ALPHA — *Robustez técnica* (grupo fechado ~50 usuários)

**Hipótese:** funciona fora do laboratório, em hardware diverso, sem quebrar nada.

- **Entra:** cobertura de sensores (rede, FPS, por-núcleo, voltagem); mais tweaks; perfis de otimização; crash reporting opt-in; auto-update assinado; instalador.
- **Fica de fora:** monetização, IA, plugins, white-label.
- **Prioridade:** estabilidade em configs reais; telemetria de erros.
- **Saída:** crash rate < 1%; rollback success ≥ 99.9%; 0 incidentes de máquina "quebrada".

## BETA — *Encaixe de produto* (público aberto, gratuito)

**Hipótese:** usuários reais valorizam, retornam e recomendam.

- **Entra:** Game Boost v1 (perfis por jogo); Digital Twin (histórico + regressões); TkReport (PDF/HTML); onboarding; i18n (PT-BR/EN/ES).
- **Fica de fora:** Enterprise/fleet, IA cloud, marketplace de plugins.
- **Prioridade:** ativação, retenção D30, NPS.
- **Saída:** D30 > 30%; NPS ≥ 50; aha-moment validado.

## v1.0 — *Comercial / monetização*

**Hipótese:** o valor sustenta assinatura paga.

- **Entra:** Licenciamento Free/Pro/Studio (+ base para Enterprise); faturamento (Stripe + PIX/boleto); relatórios white-label (Studio); hardening + **auditoria de segurança externa**; Microsoft Store; suporte.
- **Fica de fora:** TkAI, plugins de terceiros, console de fleet completo.
- **Prioridade:** conversão Free→Pro, confiança comercial, segurança.
- **Saída:** GA; conversão ≥ 3%; churn < 5%; auditoria sem achados críticos.

## v2.0 — *Inteligência + ecossistema*

**Hipótese:** IA e extensibilidade aumentam retenção, LTV e defensabilidade.

- **Entra:** TkAI (L1→L2→L3 conforme [06](06-TKAI-IA.md)); SDK de plugins + sandbox WASM + marketplace inicial; Enterprise (fleet/console, GPO/MDM, SSO); sync multi-máquina opt-in.
- **Fica de fora (futuro):** macOS/Linux; integrações OEM profundas; B2B vertical.
- **Prioridade:** moat (Twin+IA+plugins), expansão B2B, LTV.
- **Saída:** TkAI ativo; 1º plugin de terceiro publicado; 1 contrato Enterprise.

## Quadro-resumo

| Estágio | Foco | Público | Monetiza? | Métrica-chave |
|---|---|---|:--:|---|
| MVP | ciclo de valor | interno | ✗ | rollback confiável |
| Alpha | robustez | ~50 | ✗ | crash rate < 1% |
| Beta | product-fit | aberto | ✗ | D30 > 30%, NPS ≥ 50 |
| v1.0 | comercial | geral | ✅ | conversão ≥ 3% |
| v2.0 | IA + ecossistema | geral + B2B | ✅✅ | LTV, moat |

## Princípios de priorização

1. **Confiança antes de recurso** — rollback e estabilidade nunca cedem a prazo.
2. **Medir antes de escalar** — não monetizar sem product-fit comprovado.
3. **Segurança como gate** — auditoria externa bloqueia o 1.0.
4. **Wedge gamer primeiro** — foco antes de amplitude.
