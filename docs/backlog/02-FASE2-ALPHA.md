# Fase 2 — Alpha

**Meta:** robustez em hardware diverso + primeiras otimizações reais e valor de uso diário.
**Critério de saída:** crash rate < 1%, rollback success ≥ 99.9%, 0 incidentes de "PC quebrado".

Epics: E08 Otimizações reais · E09 Game Boost · E10 Relatórios · E11 Histórico.

---

## TK-E08 · Motor de Otimização (otimizações reais)
> TkOptimizer sobre a saga já validada na Fase 1. Cada tweak = 1 `Optimization`.

### TK-F081 · Catálogo de otimizações seguras

#### TK-S0811 · Plataforma de tweaks (registry/serviços/power) em tk-platform-win
- **Como** engenheiro, **quero** wrappers seguros de mutação **para** que tweaks não façam shell-out.
- **Prioridade:** P0 · **Esforço:** L (8) · **Risco:** 🔴
- **Tasks:** `registry` read/write tipado com captura do valor anterior; `services` StartType/estado; `power` get/set scheme; whitelist/blacklist de itens críticos.
- **AC:** todas as mutações capturam estado anterior; itens críticos bloqueados.
- **Dependências:** TK-S0621 (saga), TK-S0611 (snapshot)
- **DoD+:** testes em VM por tipo de alvo.

#### TK-S0812 · Catálogo inicial (~10 tweaks `safe`)
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🔴
- **Tasks:** energia (alto desempenho), efeitos visuais, indexação, startup off, limpeza temp (→quarentena), ajustes de telemetria do Windows; cada um com `preview/capture/apply/verify/revert` + `risk_level`.
- **AC:** cada tweak previsualiza, aplica via saga, verifica e reverte; nada sem snapshot.
- **Dependências:** TK-S0811

#### TK-S0813 · SafetyGuard & confirmação por risco
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🔴
- **Tasks:** bloquear combinações perigosas; exigir confirmação explícita p/ `moderate`/`advanced`; kill switch por feature flag.
- **AC:** tweak avançado exige confirmação; tweak marcado como instável pode ser desativado remotamente.
- **Dependências:** TK-S0812

### TK-F082 · Otimização guiada + medição

#### TK-S0821 · Fluxo otimizar→medir (benchmark integrado)
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** plano recomendado a partir do diagnóstico; benchmark before→apply→after; exibir ganho.
- **AC:** usuário aplica plano e vê ganho real medido.
- **Dependências:** TK-S0812, TK-S0512, TK-S0331

#### TK-S0822 · Otimização de rede (TCP/DNS) — opt-in
- **Prioridade:** P2 · **Esforço:** M (5) · **Risco:** 🔴
- **Tasks:** autotuning TCP, DNS; tudo reversível; aviso de impacto.
- **AC:** parâmetros aplicados e revertidos; sem perda de conectividade nos testes.
- **Dependências:** TK-S0811

---

## TK-E09 · Game Boost

### TK-F091 · Detecção de jogo

#### TK-S0911 · GameDetector (foreground/fullscreen + assinaturas)
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:** detectar foreground fullscreen (ETW/win event); base de assinaturas de jogos; eventos `gameboost:*`.
- **AC:** detecta início/fim de jogo conhecido com baixa taxa de falso positivo.
- **Dependências:** TK-S0211

### TK-F092 · Aplicação e perfis

#### TK-S0921 · Aplicar boost reversível (power/prioridade/suspensão)
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🔴
- **Tasks:** snapshot→trocar power plan, elevar prioridade do jogo, suspender processos configurados; restore por hook de saída + watchdog.
- **AC:** ao fechar o jogo, 100% do estado é restaurado, mesmo após crash do jogo (watchdog).
- **Dependências:** TK-S0911, TK-S0811
- **DoD+:** teste de restauração após kill abrupto do jogo.

#### TK-S0922 · Perfis por jogo (CRUD)
- **Prioridade:** P2 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** `game_profiles` CRUD; UI de configuração; defaults curados.
- **AC:** usuário cria/edita perfil por jogo; persiste.
- **Dependências:** TK-S0921

---

## TK-E10 · Relatórios

### TK-F101 · Geração de relatórios

#### TK-S1011 · Render HTML (resumo, score, gargalos, antes/depois)
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** template HTML premium; injetar diagnosis+score+benchmarks; gráficos estáticos.
- **AC:** relatório HTML completo e legível gerado a partir de um run.
- **Dependências:** TK-S0331, TK-S0512

#### TK-S1012 · Exportar PDF
- **Prioridade:** P2 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** HTML→PDF (printpdf/wkhtmltopdf); salvar em `reports`; abrir.
- **AC:** PDF fiel ao HTML, paginado.
- **Dependências:** TK-S1011

---

## TK-E11 · Histórico (base do Digital Twin)

### TK-F111 · Série temporal & visualização

#### TK-S1111 · Persistência histórica + rollups (m1, 1 ano)
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** consolidar rollups m1; retenção 1 ano; consultas por janela/contexto.
- **AC:** histórico consultável por período; tamanho do DB controlado.
- **Dependências:** TK-S0122

#### TK-S1112 · Tela Histórico (timeline + marcos)
- **Prioridade:** P2 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** linha do tempo de score; marcos (otimizações/updates); sparklines de tendência.
- **AC:** usuário vê evolução do score e eventos sobrepostos.
- **Dependências:** TK-S1111, TK-S0711
