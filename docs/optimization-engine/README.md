# TkOptimization Engine — Engenharia de Performance (Fase 2)

> Não é um "otimizador de tweaks". É uma **plataforma que prova cientificamente**
> se uma alteração melhora a máquina — ou reverte se não melhorar.

## Princípio fundamental (inegociável)
- **Nunca afirmar ganho sem benchmark.**
- **Nunca aplicar sem rollback.**
- **Nunca alterar sem auditoria.**
- **Nunca esconder impacto.**

Toda otimização passa, obrigatoriamente, pelo **loop fechado**:
```
Detectar → Justificar → Snapshot → Benchmark ANTES → Aplicar →
Benchmark DEPOIS → Confidence Engine → Comparar → Relatório → Manter ou Reverter
```
Sem evidência válida → **resultado = inconclusivo** (e a alteração é revertida por padrão).

## A grande sacada arquitetural: reúso, não reinvenção
O Engine **orquestra** componentes já construídos e validados:
| Capacidade | Já existe em | Papel no Engine |
|---|---|---|
| Snapshot/rollback/integridade/auditoria | `tk-rollback::ProtectionService` | passo Snapshot e Reverter |
| Benchmark + percentis + 1%/0.1% low | `tk-perflab` | Benchmark Antes/Depois |
| Noise floor + confiança + variância | `tk-perflab::confidence` | decidir se o ganho é real |
| Comparação com margem dinâmica | `tk-perflab::compare` | ganho/perda/instável |
| Detecção de gargalo | `tk-perflab::detect` + `tk-analyzer` | Advisor |
| TkSpeed Score | `tk-analyzer::score` | score antes/depois |
| Acesso seguro ao SO (HKCU/serviços/power) | `tk-platform-win` | Aplicar (reversível) |

→ O Engine é o **fecho do ciclo**. Isso o torna implementável de forma incremental e segura.

## Índice
| Doc | Conteúdo | Entregável |
|---|---|---|
| [01 · Arquitetura](01-ARQUITETURA.md) | 7 módulos + crate `tk-optimize` + integração | 1 |
| [02 · Estrutura de Dados](02-ESTRUTURA-DADOS.md) | tipos + migração `0004` | 3 |
| [03 · Catálogo Inicial](03-CATALOGO.md) | otimizações + classificação SAFE/MODERATE/ADVANCED/EXPERIMENTAL | 4 |
| [04 · Pipelines](04-PIPELINES.md) | execução · validação · rollback | 5,6,7 |
| [05 · Advisor & Perfis](05-ADVISOR-PROFILES.md) | inteligência de decisão + 7 perfis | (parte de 1) |
| [06 · Backlog & Slices](06-BACKLOG-SLICES.md) | backlog executável + plano por slices + QA | 2,8 |

## Limites desta fase
Apenas **arquitetura, catálogo e pipeline** agora. Implementação vem em **slices incrementais e validáveis** (ver doc 06). Nenhuma alteração destrutiva, nunca.
