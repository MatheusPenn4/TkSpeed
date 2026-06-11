# 02 · Architecture Decision Records (ADR)

> Formato: Contexto · Decisão · Vantagens · Riscos · Mitigação · Alternativas descartadas · Status.

---

## ADR-001 · Linguagem de núcleo: **Rust**

- **Contexto:** o produto altera o SO (Registry, serviços, energia, processos). Erros de memória ou concorrência podem corromper a máquina do usuário.
- **Decisão:** Rust para todo o núcleo e adapters de plataforma.
- **Vantagens:** segurança de memória sem GC; concorrência sem data races; performance nativa (footprint baixo); excelente FFI com a Windows API; ecossistema maduro (tokio, serde, sqlx).
- **Riscos:** curva de aprendizado; mercado de contratação menor; tempo de compilação.
- **Mitigação:** workspace multi-crate (build incremental); padrões/documentação internos; contratar 1–2 sêniores Rust como âncora.
- **Alternativas descartadas:** **C#/.NET** (runtime maior, menos controle de baixo nível), **C++** (poder igual sem as garantias de segurança), **Go** (GC e FFI Windows menos ergonômico para sensores).
- **Status:** Aceito.

---

## ADR-002 · Shell desktop: **Tauri 2**

- **Contexto:** precisamos de UI premium moderna + acesso profundo ao SO + footprint mínimo (a ferramenta não pode pesar no PC que promete acelerar).
- **Decisão:** Tauri 2 (WebView2 no Windows) com backend Rust.
- **Vantagens:** binário ~10–20× menor que Electron; WebView2 já presente no Win11; backend Rust nativo; modelo de permissões (capabilities) e CSP fortes; auto-updater integrado.
- **Riscos:** dependência do WebView2; APIs mais novas que Electron; menos exemplos prontos.
- **Mitigação:** bootstrap do WebView2 no instalador; abstrair IPC em camada própria; investir em testes e2e.
- **Alternativas descartadas:** **Electron** (peso e RAM contradizem a proposta de valor), **WPF/.NET MAUI** (UI menos moderna/portável, dificulta o design premium), **Qt** (licenciamento e DX inferiores ao stack web para este time).
- **Status:** Aceito.

---

## ADR-003 · Persistência local: **SQLite (WAL)**

- **Contexto:** dados local-first (telemetria histórica, snapshots, scores, licença) sem servidor obrigatório.
- **Decisão:** SQLite com WAL via `sqlx`.
- **Vantagens:** transacional (essencial para snapshots/rollback); SQL para o Digital Twin; zero servidor; maduro e confiável; arquivo único portável.
- **Riscos:** crescimento da série temporal; concorrência de escrita.
- **Mitigação:** downsampling/rollups (s1→s10→m1) e retenção; WAL + pool; `VACUUM` periódico; (futuro) DuckDB para analytics pesado do Twin.
- **Alternativas descartadas:** **arquivos JSON puros** (sem transação/consulta), **sled/RocksDB** (sem SQL para o Twin), **Postgres local** (operacionalmente pesado para desktop).
- **Status:** Aceito.

---

## ADR-004 · **Clean Architecture**

- **Contexto:** produto de vida longa, multi-módulo, com regras de negócio que não podem depender de detalhes de plataforma.
- **Decisão:** camadas Domain ← Application ← Infra/Adapters, com dependências apontando para dentro.
- **Vantagens:** testabilidade do domínio sem SO; troca de adapters (WMI↔LHM) sem tocar regras; longevidade.
- **Riscos:** over-engineering; boilerplate em features simples.
- **Mitigação:** aplicar pragmaticamente (não criar camada onde não há regra); vertical slices no frontend.
- **Alternativas descartadas:** **arquitetura em camadas tradicional acoplada** (rápida no início, dívida cara depois), **transaction script puro** (não escala em complexidade).
- **Status:** Aceito.

---

## ADR-005 · **Domain-Driven Design (DDD)**

- **Contexto:** domínios distintos (monitoramento, otimização, rollback, benchmark, game boost) com linguagens próprias.
- **Decisão:** bounded contexts isolados em crates, com linguagem ubíqua por contexto.
- **Vantagens:** equipes paralelas sem colisão; modelos coesos; fronteiras claras para plugins futuros.
- **Riscos:** complexidade de coordenação entre contextos; duplicação de tipos.
- **Mitigação:** `tk-contracts` como contexto compartilhado mínimo; eventos para integração.
- **Alternativas descartadas:** **monólito sem fronteiras** (vira big ball of mud), **microsserviços** (absurdo para um desktop).
- **Status:** Aceito.

---

## ADR-006 · **Saga Pattern** para otimizações

- **Contexto:** uma otimização é um conjunto de mutações no SO que precisa ser tudo-ou-nada (do ponto de vista do usuário) e reversível.
- **Decisão:** cada plano roda como saga: `snapshot → apply → verify → commit | compensate`.
- **Vantagens:** consistência (sem estado parcial); compensação automática em falha; auditável; idempotente.
- **Riscos:** complexidade de implementar `revert` correto para cada tweak; ordem de compensação.
- **Mitigação:** trait `Optimization` obriga `capture/verify/revert`; testes de idempotência em VM; ordem inversa garantida pelo orquestrador.
- **Alternativas descartadas:** **transação distribuída/2PC** (não aplicável ao SO), **aplicar sem compensação** (inaceitável dado o risco).
- **Status:** Aceito.

---

## ADR-007 · **Rollback obrigatório** (invariante de produto)

- **Contexto:** confiança é o principal diferencial; concorrentes perderam reputação por mudanças irreversíveis.
- **Decisão:** **nenhuma** mutação ocorre sem snapshot+log+plano de rollback. A saga recusa-se a aplicar se o snapshot falhar.
- **Vantagens:** confiança do usuário; suporte mais barato (reversão resolve incidentes); diferenciação de marca; defesa legal.
- **Riscos:** custo de armazenamento de snapshots; overhead por operação.
- **Mitigação:** snapshots enxutos (só o delta), quarentena para arquivos, retenção configurável, integração com Restore Point para alto impacto.
- **Alternativas descartadas:** **rollback opcional** (mina a proposta de valor), **somente Restore Point do Windows** (granularidade e velocidade insuficientes).
- **Status:** Aceito (inegociável).

---

## ADR-008 · Telemetria **local por padrão** *(decisão de suporte)*

- **Decisão:** dados ficam no dispositivo; qualquer envio é opt-in e anonimizado.
- **Vantagens:** privacidade como diferencial; conformidade LGPD/GDPR simplificada; confiança.
- **Riscos:** menos dados para melhorar o produto.
- **Mitigação:** analytics agregado opt-in com incentivo claro.
- **Status:** Aceito.

---

## ADR-009 · Tipagem ponta-a-ponta via **`ts-rs`** *(decisão de suporte)*

- **Decisão:** tipos cruzando IPC vivem em `tk-contracts` e geram `.ts` automaticamente.
- **Vantagens:** zero drift entre Rust e TS; refatoração segura.
- **Riscos:** etapa extra no build.
- **Mitigação:** integrar geração ao pipeline; fallback de tipos manuais no início.
- **Status:** Aceito.
