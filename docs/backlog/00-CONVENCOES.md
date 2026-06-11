# 00 · Convenções do Backlog

## Esquema de IDs

| Tipo | Prefixo | Exemplo |
|---|---|---|
| Epic | `TK-E` | `TK-E01` |
| Feature | `TK-F` | `TK-F011` |
| User Story | `TK-S` | `TK-S0112` |
| Task técnica | `TK-T` | `TK-T01121` |

Hierarquia codificada no número: `TK-S0112` = Epic 01 → Feature 1 → Story 2.

## Prioridade (MoSCoW + ordem)

| Nível | Significado |
|---|---|
| **P0** | Bloqueador / caminho crítico — sem isto, nada avança |
| **P1** | Must — essencial para a fase |
| **P2** | Should — importante, mas adiável |
| **P3** | Could — desejável |

## Esforço (T-shirt → Story Points)

| Tamanho | Pontos | Referência |
|---|--:|---|
| XS | 1 | < ½ dia |
| S | 2 | ~1 dia |
| M | 5 | 2–3 dias |
| L | 8 | ~1 semana |
| XL | 13 | 2+ semanas (quebrar se possível) |

## Risco

| Nível | Critério |
|---|---|
| 🟢 Baixo | Tecnologia conhecida, sem dependência externa |
| 🟡 Médio | Integração com SO/3rd-party, incerteza moderada |
| 🔴 Alto | Acesso privilegiado, mutação de SO, segurança, incerteza técnica |

## Definição de Pronto (DoD) — global

Toda User Story só é "Done" quando:

1. ✅ Código revisado (PR aprovado por ≥1 revisor).
2. ✅ Testes automatizados cobrindo o caminho feliz + 1 erro (unit no domínio; integração em adapters).
3. ✅ Sem regressão no **health budget** (CPU idle < 1%, RAM < 150 MB) — verificado no CI.
4. ✅ Tipos cruzando IPC gerados via `ts-rs` (sem drift Rust↔TS).
5. ✅ Telemetria de erro/log estruturado adicionada onde aplicável.
6. ✅ Documentação/changelog atualizados.
7. ✅ Critérios de aceite (AC) da story validados em build de QA.
8. ✅ **Para qualquer story que mute o SO:** snapshot+rollback testado em VM (idempotência do revert).

> Cada story abaixo lista **AC específicos** e **DoD adicional** quando diferente do global.

## Componentes (labels)

`core` · `storage` · `monitor` · `analyzer` · `score` · `benchmark` · `rollback` · `optimizer` · `gameboost` · `report` · `history` · `licensing` · `updater` · `plugins` · `ai` · `cloud` · `enterprise` · `ui` · `design-system` · `ipc` · `ci` · `security`
