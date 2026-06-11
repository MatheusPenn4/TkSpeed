# 01 · Arquitetura — TkOptimization Engine

Novo bounded context: crate **`tk-optimize`**. Orquestra os serviços existentes;
não duplica medição nem persistência.

## 1. Posição no workspace
```
tk-optimize  (NOVO)
 ├─ depende de: tk-contracts, tk-storage, tk-platform-win,
 │              tk-rollback (ProtectionService), tk-perflab, tk-analyzer
 ├─ usado por:  src-tauri (bridge/comandos) e UI "Centro de Otimizações"
 └─ NÃO é dependido por nenhum crate de medição (fica no topo da composição)
```
> Reintroduz o papel do antigo `tk-optimizer` (removido na limpeza B5), agora
> **async** e ligado aos serviços reais — não à saga síncrona descartada.

## 2. Os 7 módulos (dentro de `tk-optimize`)

```
                         TkOptimizationEngine (facade / loop fechado)
        ┌──────────────┬───────────────┬──────────────┬───────────────┐
        ▼              ▼               ▼              ▼               ▼
   Catalog        Executor        Validator       Evidence        Advisor
 (o que existe) (aplica+snapshot)(bench+confiança)(prova/relatório)(o que fazer)
        └───────────────────────── Profiles (bundles rastreáveis) ───┘
```

### TkOptimizationEngine
Fachada que executa o **pipeline obrigatório** (doc 04). API central:
`run(optimization_id, opts) -> OptimizationRun` e `run_profile(profile_id) -> Vec<OptimizationRun>`.
Coordena os demais módulos e garante a ordem snapshot→bench→apply→bench→validar→decidir.

### TkOptimizationCatalog
Registro de otimizações. Cada item implementa o trait `Optimization` (1 arquivo = 1 tweak):
```rust
#[async_trait]
pub trait Optimization: Send + Sync {
    fn meta(&self) -> &OptimizationMeta;                 // metadados (doc 02)
    async fn preview(&self, ctx: &SysCtx) -> Result<Preview>;       // dry-run: o que muda
    async fn capture(&self, ctx: &SysCtx) -> Result<Vec<ReversibleAction>>; // estado p/ rollback
    async fn apply(&self, ctx: &mut SysCtx) -> Result<ApplyOutcome>;        // muta (reversível)
    async fn verify(&self, ctx: &SysCtx) -> Result<bool>;            // pós-condição
    // revert NÃO é responsabilidade do tweak: o ProtectionService restaura via snapshot.
}
```
`catalog()` devolve `Vec<Box<dyn Optimization>>`; novos tweaks são adicionados sem tocar o core.

### TkOptimizationExecutor
Aplica de forma **transacional**, reusando o `ProtectionService` (generalizado p/ múltiplos
alvos: registry HKCU, quarentena de arquivos, plano de energia, serviços):
`capture → snapshot(reason) → apply → verify`. Falha em qualquer etapa → rollback automático.
**Invariante: recusa aplicar se o snapshot falhar.**

### TkOptimizationValidator
Roda o **benchmark associado** (via `tk-perflab`) antes e depois, aplica o **Confidence Engine**
(noise floor + confiança + variância) e decide contra os `success_criteria`/`rollback_criteria`
da otimização. Saída: `Decision { Keep | Revert | Inconclusive }`.

### TkOptimizationEvidence
Monta e persiste o **registro de evidência**: score antes/depois, benchmark antes/depois,
ganho/perda real, confiança, decisão. **Sem evidência válida → `Inconclusive`** (e reverte).
Exportável (base do futuro relatório/laudo).

### TkOptimizationAdvisor
A inteligência de decisão. A partir do gargalo detectado (`tk-perflab::detect` / `tk-analyzer`),
recomenda otimizações específicas com **motivo · impacto esperado · risco · evidência prévia**
(doc 05). Não aplica nada — só recomenda.

### TkOptimizationProfiles
Bundles nomeados e **rastreáveis** (cada perfil = lista de ids do catálogo). Aplicar um perfil =
rodar o pipeline para cada otimização, com snapshot único de perfil para reversão em bloco (doc 05).

## 3. `SysCtx` e segurança
`SysCtx` abstrai o acesso ao SO (delegando a `tk-platform-win`): registry HKCU (sem UAC),
quarentena de arquivos, plano de energia, serviços. Elevação só quando o `risk_level` exigir,
via `PermissionBroker` (UAC sob demanda) — SAFE nunca eleva.

## 4. Integração com o resto do app
- **Persistência:** novas tabelas via migração `0004` (doc 02) + reúso de `snapshots`,
  `benchmark_sessions`, `audit_log`.
- **IPC/bridge:** comandos `opt_catalog`, `opt_advise`, `opt_preview`, `opt_run`, `opt_run_profile`,
  `opt_history`, `opt_rollback` (tipos em `tk-contracts`, gerados por ts-rs no futuro).
- **UI:** "Centro de Otimizações" (doc 06 / dashboard).
- **AppContext:** ganha `ctx.optimize()` (DI), como `protection()`/`perf()`.

## 5. Fluxo de alto nível
```
Advisor (gargalo → recomendações)
   ↓ usuário escolhe
Engine.run(id):
   Executor.snapshot+preview → Validator.bench_before → Executor.apply+verify
        → Validator.bench_after + Confidence → Evidence.build → Decision
        → Keep (mantém) | Revert (ProtectionService.rollback) | Inconclusive (revert)
   → persiste OptimizationRun + Evidence + auditoria
```
