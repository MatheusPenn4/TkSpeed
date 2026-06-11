# 09 · Estratégia de Rollback

> **Invariante do produto:** nada é alterado no sistema sem que exista um caminho de volta verificado. Esta é a feature de confiança central.

## 1. Conceito

Toda mutação do sistema é encapsulada numa **transação reversível**. Antes de aplicar, capturamos o estado anterior num **snapshot imutável**; em falha (ou a pedido do usuário) restauramos.

## 2. Tipos de alvo e como são revertidos

| Alvo | Captura (snapshot) | Revert |
|---|---|---|
| Registry | valor anterior (ou ausência) da chave | reescreve/remove chave |
| Serviços Windows | `StartType` + estado atual | restaura tipo de início + estado |
| Plano de energia | GUID ativo + settings alterados | reativa GUID/settings anteriores |
| Arquivos (limpeza) | move para **quarentena** (não deleta) | move de volta da quarentena |
| Startup items | estado habilitado/desabilitado | restaura estado |
| Rede/TCP | parâmetros anteriores | reaplica parâmetros |

> Arquivos **nunca** são deletados imediatamente: vão para quarentena com TTL. Só após o TTL (e sem rollback solicitado) são removidos de fato.

## 3. Saga transacional (Unit of Work)

```
begin():
   snapshot = SnapshotStore.create(reason)
   audit.log("optimize.intent", plan)

for op in plan.operations:
   before = op.capture_state()          # → snapshot_entries
   op.apply()                           # mutação real
   if !op.verify(): raise Failure

commit():
   snapshot.status = active
   audit.log("optimize.committed", result)

on Failure:                              # compensação automática
   for entry in snapshot.entries.reverse():
       entry.restore()
   snapshot.status = restored
   audit.log("optimize.rolledback", error)
```

- **Atômico por percepção:** se qualquer operação falha, todas as já aplicadas são revertidas em ordem inversa.
- **Idempotente:** `restore()` pode rodar múltiplas vezes sem efeito colateral.

## 4. Camadas de rollback

1. **Operacional (saga):** reverte um plano específico (imediato/automático em falha).
2. **Snapshot manual:** usuário escolhe um snapshot do histórico e restaura total ou granularmente (por entrada).
3. **System Restore Point:** para ações de maior impacto, integra com o ponto de restauração do Windows como rede de segurança extra.
4. **Quarentena:** recuperação de arquivos removidos por limpeza.

## 5. Integridade

- Cada snapshot tem `integrity_hash` (hash das entradas). Restauração valida o hash antes de aplicar.
- Snapshots são **imutáveis** após criação; um novo estado gera novo snapshot.
- `audit_log` registra criação, restauração e expiração.

## 6. UX de rollback (tela Rollback)

- Linha do tempo de snapshots (data, motivo, nº de mudanças, status).
- "Reverter tudo" e "Reverter item" (granular).
- Diff visual: o que estava antes vs depois.
- Quarentena com "restaurar arquivo".
- Confirmação clara; nenhuma restauração sem o usuário ver o que muda.

## 7. Garantias

- **Sem snapshot ⇒ sem aplicação.** O `OptimizationSaga` recusa-se a aplicar se o snapshot falhar.
- Snapshots persistem entre sessões e sobrevivem a reinício.
- Política de retenção configurável; mínimo dos últimos N snapshots sempre preservado.
