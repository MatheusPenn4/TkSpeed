# 08 · Matriz de Riscos

> Escala: Probabilidade (B/M/A) × Impacto (B/M/A). Severidade = combinação. Cada risco tem dono e mitigação.

## Riscos técnicos

| ID | Risco | Prob | Impacto | Mitigação |
|---|---|:--:|:--:|---|
| T1 | Otimização quebra o SO do usuário | M | **A** | Saga com snapshot+rollback obrigatório; whitelist de itens críticos; testes em VM; verify pós-condição; `risk_level` + confirmação |
| T2 | Cobertura de sensores falha em hardware diverso | A | M | Múltiplos adapters (LHM/WMI/PDH) com fallback; telemetria de falhas; matriz de testes de hardware |
| T3 | Dependência do WebView2/Tauri (breaking changes) | M | M | Abstrair IPC; fixar versões; testes e2e; acompanhar releases |
| T4 | Curva/escassez de Rust atrasa entregas | M | M | Âncoras sêniores; padrões internos; escopo modular |
| T5 | Crescimento descontrolado do DB de telemetria | M | M | Downsampling/rollups; retenção; VACUUM; monitor de tamanho |
| T6 | Falha no `revert` de um tweak específico | B | A | Trait obriga revert; testes de idempotência; quarentena; Restore Point como rede |

## Riscos legais & de conformidade

| ID | Risco | Prob | Impacto | Mitigação |
|---|---|:--:|:--:|---|
| L1 | LGPD/GDPR (dados de hardware/uso) | M | A | Telemetria local; envio só opt-in/anonimizado; DPA; política de privacidade clara |
| L2 | Responsabilidade por dano ao PC do usuário | B | A | EULA com limitação; rollback como defesa real; logs/auditoria; seguro E&O |
| L3 | Licenças de dependências (LHM é MPL/GPL?) | M | M | Auditoria de licenças (`cargo deny`); isolar componentes copyleft via processo/serviço separado; alternativas próprias |
| L4 | Marca/Trademark "TkSpeed" | M | M | Busca e registro de marca cedo; domínios |
| L5 | Exigências de loja (Microsoft Store) | B | M | Seguir políticas; build assinado; revisão antecipada |

## Riscos de segurança

| ID | Risco | Prob | Impacto | Mitigação |
|---|---|:--:|:--:|---|
| S1 | Abuso do componente elevado | B | **A** | Superfície fechada/validada; sem comandos arbitrários; APIs nativas, não shell-out |
| S2 | Update malicioso / supply chain | B | A | Assinatura Authenticode + updater assinado; SBOM; `cargo audit`; reprodutibilidade |
| S3 | Vazamento de telemetria | B | M | Local por padrão; criptografia em repouso de dados sensíveis; opt-in |
| S4 | Plugins maliciosos (Fase 5) | M | A | Sandbox WASM; capabilities aprovadas; assinatura; curadoria; revogação |
| S5 | Pirataria/forja de licença | M | M | Assinatura Ed25519; valor server-gated; device binding (ver [03](03-LICENCIAMENTO.md)) |

## Riscos de performance

| ID | Risco | Prob | Impacto | Mitigação |
|---|---|:--:|:--:|---|
| P1 | A ferramenta pesa no PC (ironia fatal) | M | A | Orçamento rígido (<1% CPU/<150MB); coleta assíncrona; gate de health no CI |
| P2 | Overhead do monitoramento contínuo em notebooks (bateria) | M | M | Modo econômico; cadências adaptativas; pausar em bateria |
| P3 | Benchmark inconsistente entre versões | M | M | Suites determinísticas e versionadas; comparar só mesma versão |

## Riscos comerciais & de mercado

| ID | Risco | Prob | Impacto | Mitigação |
|---|---|:--:|:--:|---|
| C1 | Baixa conversão Free→Pro | M | A | Gating bem calibrado; aha-moment; trials; provas de valor (antes/depois) |
| C2 | Percepção "é só mais um otimizador" | M | A | Posicionamento "plataforma inteligente"; reputação de honestidade; conteúdo |
| C3 | Concorrente grande (OEM/Avast) copia | M | M | Velocidade; moat de Twin+IA+UX; comunidade/plugins |
| C4 | CAC alto / canais caros | M | M | Wedge gamer viral; afiliados/criadores; SEO de dores; PLG |
| C5 | Dependência de poucos fundadores | M | A | Documentação (este repo!); bus factor; contratação faseada |
| C6 | Reputação manchada por 1 incidente de "PC quebrado" | B | **A** | Rollback robusto; beta longo; suporte rápido; comunicação transparente |

## Top 5 riscos a vigiar (heatmap)

1. **T1** — quebrar o SO (mitigado pela arquitetura de rollback, mas impacto catastrófico).
2. **C2** — posicionamento confundido com otimizador comum.
3. **S1** — abuso do componente elevado.
4. **P1** — footprint da própria ferramenta.
5. **C1** — conversão insuficiente.

## Plano de contingência (incidente grave)

- **Kill switch de tweak**: catálogo versionado permite desabilitar remotamente um tweak problemático (via feature flag/lista).
- **Rollback assistido em massa**: orientação + ferramenta para reverter via snapshot/Restore Point.
- **Comunicação**: post-mortem público transparente — converter incidente em prova de confiabilidade.
