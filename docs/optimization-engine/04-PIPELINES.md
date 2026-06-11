# 04 · Pipelines (Execução · Validação · Rollback)

## 1. Pipeline de EXECUÇÃO (obrigatório, sem atalhos)

```
Engine.run(optimization_id):
 0. carrega meta do Catálogo; checa risco/elevação (PermissionBroker; SAFE não eleva)
 1. PREVIEW   → preview(): mostra exatamente o que muda (dry-run); usuário confirma se MODERATE+
 2. CAPTURE   → capture(): lê estado atual (ReversibleAction[])
 3. SNAPSHOT  → ProtectionService cria snapshot(reason=optimization_id) + entries  ── SEM ISSO, ABORTA
                audit("optimize.intent")
 4. BENCH ANTES (se Validation=Benchmark/Score) → tk-perflab roda a suite associada → before_session
 5. APPLY     → apply(): muta via tk-platform-win;  audit("optimize.applied")
 6. VERIFY    → verify(): confere pós-condição. Falhou? → vai direto ao Rollback (passo R)
 7. BENCH DEPOIS → mesma suite, mesmas condições → after_session
 8. VALIDAR/DECIDIR (pipeline de validação, abaixo)
 9. RELATÓRIO → Evidence.build(...) persiste OptimizationRun + evidence_json
10. MANTER ou REVERTER conforme a decisão
```
- A ordem **Snapshot antes do Bench Antes** garante reversão mesmo se o benchmark falhar.
- SAFE com `Validation=SpaceFreed` pula 4/7 e mede bytes liberados (evidência honesta).

## 2. Pipeline de VALIDAÇÃO (o juiz)

```
Validator.decide(before_session, after_session, meta):
 a. compare = tk-perflab::compare(before, after, noise_profile)   // margem dinâmica + Confidence Engine
 b. se !compare.reliable           → Decision::Inconclusive  (medição instável)
 c. senão, avalia success_criteria sobre a métrica primária da suite:
      - ganho real (verdict=Gain, delta>margem)  → Decision::Keep
      - perda real (verdict=Loss)                → Decision::Revert
      - sem alteração significativa               → Decision::Revert (não vale manter o risco/mudança)
 d. score_before/after (TkSpeed Score) entram na evidência (contexto)
```
Regras-chave (honestidade):
- **Inconclusivo nunca é "ganho".** Medição instável/contaminada (térmica) → não decide a favor.
- **"Sem alteração" → reverte por padrão** (não manter alteração que não comprovou benefício).
- Para `SpaceFreed`/`None`, o critério é o efeito declarado (bytes liberados / relatório gerado),
  e a evidência **não** afirma ganho de performance.

## 3. Pipeline de ROLLBACK (rede de segurança)

```
Acionado por: verify() falhou | Decision=Revert | Decision=Inconclusive | erro em qualquer etapa | usuário
Rollback(snapshot_id):
 1. ProtectionService valida integridade (hash) do snapshot          ── inválido → aborta restauração
 2. restaura cada ReversibleAction em ordem inversa:
      RegistryHkcu → reescreve/remove; FileQuarantine → restaura da quarentena;
      PowerPlan → reativa GUID anterior; ServiceStart → restaura StartType
 3. verifica restauração (lê de volta == estado original)
 4. status=reverted; audit("optimize.rolledback")
```
- **Idempotente** e **verificado** (igual ao rollback já validado no MVP).
- Arquivos nunca são apagados na aplicação — vão para **quarentena**; rollback os traz de volta.
- Para ADVANCED de maior risco, integra **ponto de restauração do Windows** como camada extra.

## 4. Aplicação de PERFIL (lote)
```
run_profile(profile_id):
 - snapshot único "profile:<nome>" cobrindo todos os itens
 - para cada otimização do perfil: roda o pipeline de execução
 - decisão por item (Keep/Revert) + relatório agregado do perfil
 - "Reverter perfil" restaura o snapshot de perfil inteiro de uma vez
```

## 5. Estados possíveis de um `OptimizationRun`
`applied` (mutou, aguardando validação) → `kept` (ganho comprovado) | `reverted` (perda/sem
alteração/erro) | `inconclusive` (medição não confiável → revertido) | `failed` (apply/snapshot falhou).

## 6. Garantias do pipeline
- Sem snapshot ⇒ sem aplicação.
- Sem benchmark válido ⇒ sem alegação de ganho (inconclusivo).
- Toda transição é auditada (`audit_log`).
- Impacto sempre exibido com **margem de erro e confiança** (nunca número pelado).
