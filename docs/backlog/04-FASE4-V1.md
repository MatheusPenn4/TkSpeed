# Fase 4 — v1.0+ / Ecossistema (Plugins, Marketplace, Digital Twin)

**Meta:** extensibilidade e inteligência histórica como moat.
**Critério de saída:** 1º plugin de terceiro publicado; Digital Twin detectando regressões reais.

Epics: E15 SDK de Plugins & Sandbox · E16 Marketplace · E17 Digital Twin.

> Ver [05-PLUGINS](../business/05-PLUGINS.md) e [14-DIGITAL-TWIN](../14-DIGITAL-TWIN.md).

---

## TK-E15 · SDK de Plugins & Sandbox

### TK-F151 · Runtime de plugins (WASM)

#### TK-S1511 · Host API + sandbox WASM + capabilities
- **Prioridade:** P1 · **Esforço:** XL (13) · **Risco:** 🔴
- **Tasks:** runtime WASM; Host API capability-gated (`metrics.read`, `optimize.*`, `analyze.report`, `report.render`); plugin propõe intenção → host valida + snapshot.
- **AC:** plugin roda isolado; mutação só via saga com snapshot; capability negada bloqueia chamada.
- **Dependências:** TK-S0621, TK-S0811
- **DoD+:** pentest do sandbox; plugin não acessa SO diretamente.

#### TK-S1512 · Manifesto + assinatura + carregamento dinâmico
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🔴
- **Tasks:** `plugin.toml` (capabilities/risk/api_version); validar assinatura Ed25519; `ModuleRegistry` carrega/descarrega; níveis de confiança; revogação.
- **AC:** plugin assinado carrega; não-assinado/revogado é bloqueado; usuário aprova capabilities.
- **Dependências:** TK-S1511

### TK-F152 · SDK & ferramentas para terceiros

#### TK-S1521 · `tkspeed-plugin-sdk` + CLI + simulador
- **Prioridade:** P2 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:** SDK (tipos Host API, macros de manifesto); CLI `new/build/test/sign/publish`; simulador sobre telemetria gravada; docs + exemplos.
- **AC:** dev externo cria, testa e assina um plugin "hello optimization" seguindo a doc.
- **Dependências:** TK-S1512

---

## TK-E16 · Marketplace

### TK-F161 · Distribuição de plugins

#### TK-S1611 · Catálogo + instalação in-app
- **Prioridade:** P2 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:** backend de catálogo; busca/instalação/atualização in-app; curadoria/revisão; ratings.
- **AC:** usuário descobre, instala e atualiza plugin pela UI.
- **Dependências:** TK-S1512

#### TK-S1612 · Revenue share + publisher portal
- **Prioridade:** P3 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:** onboarding de publisher; pagamentos/split; relatórios de venda.
- **AC:** publisher publica pago e recebe split.
- **Dependências:** TK-S1611, TK-S1313

---

## TK-E17 · Digital Twin (completo)

### TK-F171 · Baseline & regressões

#### TK-S1711 · Baseline móvel + detecção de anomalia
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:** baseline por contexto (idle/load/game) com média/desvio; flag de anomalia (k·desvio).
- **AC:** desvio significativo gera insight; baseline atualiza por janela.
- **Dependências:** TK-S1111

#### TK-S1712 · Correlação evento↔métrica (ex.: regressão de driver)
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:** ligar `audit_log`/inventário a séries; comparar janelas pré/pós-evento no mesmo contexto; gerar insight ("driver X → −8% FPS").
- **AC:** atualização de driver seguida de queda gera insight correto com evidência.
- **Dependências:** TK-S1711, TK-S0221

#### TK-S1713 · Cards de insight na tela Histórico
- **Prioridade:** P2 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** UI de insights (regressão/melhoria sustentada); ação sugerida (sempre reversível).
- **AC:** insights exibidos com evidência e ação.
- **Dependências:** TK-S1712, TK-S1112
