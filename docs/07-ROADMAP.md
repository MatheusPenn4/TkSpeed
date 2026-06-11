# 07 · Roadmap

> Cada fase tem critério de saída ("Definition of Done") explícito. Datas são relativas ao início do desenvolvimento (T0).

## FASE 1 — MVP (T0 → T0+3 meses)

**Objetivo:** provar o ciclo completo *monitorar → diagnosticar → otimizar com rollback → medir*.

- [x] Scaffolding Tauri + Rust workspace + React + SQLite.
- TkCore (bootstrap, DI, permissões, event bus).
- TkMonitor: CPU, RAM, GPU básica, storage, temperaturas (LHM + WMI + PDH).
- TkAnalyzer: detecção de gargalo CPU/RAM/Storage + startup analyzer.
- TkOptimizer: ~10 tweaks seguros (energia, visuais, startup, temp cleanup).
- TkRollback: snapshot + restore funcional (Registry/serviços/power).
- TkBenchmark: CPU + RAM + SSD básicos.
- TkSpeed Score v1.
- Telas: Dashboard, Monitoramento, Central de Diagnóstico, Rollback, Settings.
- **DoD:** usuário aplica otimização, mede ganho e consegue reverter 100%.

## FASE 2 — Beta (T0+3 → T0+6 meses)

**Objetivo:** robustez, cobertura e UX premium completa.

- TkMonitor: rede, FPS (ETW/PresentMon-like), por-núcleo, voltagem.
- TkAnalyzer: drivers desatualizados, serviços, conflitos, bottleneck térmico/rede/GPU.
- TkOptimizer: catálogo ampliado (rede/TCP, ajustes gamer), perfis de otimização.
- TkGameBoost v1 (detecção + perfis por jogo).
- TkReport (PDF/HTML).
- Digital Twin (histórico + comparações, ex. regressão pós-driver).
- Auto-updater assinado, instalador MSI/NSIS.
- Telemetria local + crash reporting opt-in.
- **DoD:** beta público fechado, < 1% crash rate, ciclo de otimização confiável em 20+ configs reais.

## FASE 3 — Versão Comercial 1.0 (T0+6 → T0+9 meses)

**Objetivo:** monetização e escala.

- Licenciamento Free/Pro/Studio (offline-first + ativação online).
- Onboarding, perfis de desempenho, agendamento.
- Relatórios white-label (Studio).
- Hardening de segurança + auditoria externa.
- Localização (PT-BR, EN, ES).
- **DoD:** 1.0 GA na loja/site, gateway de pagamento, suporte.

## FASE 4 — IA (T0+9 → T0+14 meses)

**Objetivo:** TkAI assistente.

- Motor de explicação ("Por que meu FPS caiu?") sobre o Digital Twin.
- Modelo local-first (SLM) + backend remoto opt-in.
- Recomendações preditivas e detecção de regressão automática.
- **DoD:** assistente responde top-20 perguntas com correlação ao histórico real.

## FASE 5 — Ecossistema (T0+14+)

**Objetivo:** plataforma extensível.

- SDK de **plugins** (otimizações e coletores de terceiros, sandbox WASM).
- Sincronização multi-máquina opcional (cloud opt-in).
- Marketplace de perfis/plugins.
- Integrações (Discord, overlays, OEM partnerships).
- API/CLI para fleets corporativas.
- **DoD:** primeiro plugin de terceiro publicado; 1 parceria OEM/B2B.

## Marcos transversais

- **Qualidade:** testes desde a Fase 1 (unit no domínio, integração nos adapters, e2e do ciclo de otimização em VM).
- **Segurança:** revisão a cada fase; auditoria externa antes do 1.0.
- **Performance da ferramenta:** orçamento de < 1% CPU em idle, < 150 MB RAM.
