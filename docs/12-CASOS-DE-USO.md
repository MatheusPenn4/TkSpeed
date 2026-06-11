# 12 · Casos de Uso

Formato: **Ator → Objetivo → Fluxo → Garantias**.

## UC-01 · Diagnóstico completo

- **Ator:** Power user.
- **Objetivo:** entender o estado do PC.
- **Fluxo:** Dashboard → "Análise Completa" → TkAnalyzer coleta telemetria + drivers + serviços + startup → exibe Central de Diagnóstico com gargalos (problema/impacto/solução) + TkSpeed Score.
- **Garantias:** somente leitura; nada é alterado.

## UC-02 · Otimização segura com medição

- **Ator:** Gamer.
- **Objetivo:** ganhar desempenho sem risco.
- **Fluxo:** Diagnóstico → seleciona plano recomendado → `preview` mostra mudanças → confirma → benchmark "antes" → saga aplica (snapshot+log) → benchmark "depois" → mostra ganho %.
- **Garantias:** snapshot criado; rollback disponível; falha = revert automático.

## UC-03 · Reverter uma otimização

- **Ator:** Usuário arrependido.
- **Objetivo:** voltar ao estado anterior.
- **Fluxo:** Rollback → escolhe snapshot → vê diff → "Reverter tudo" ou item específico → estado restaurado.
- **Garantias:** integridade validada por hash; auditado.

## UC-04 · Game Boost automático

- **Ator:** Gamer.
- **Objetivo:** melhor experiência sem mexer manualmente.
- **Fluxo:** abre o jogo → GameDetector identifica → aplica GameProfile (suspende apps, troca power plan, prioriza jogo) → ao fechar, restaura tudo.
- **Garantias:** estado restaurado por hook de saída + watchdog; tudo reversível.

## UC-05 · Detecção de regressão (Digital Twin)

- **Ator:** Entusiasta.
- **Objetivo:** descobrir por que o PC "piorou".
- **Fluxo:** Histórico → linha do tempo → TkSpeed nota "Após atualização do driver NVIDIA, FPS médio caiu 8%" correlacionando evento + telemetria.
- **Garantias:** baseado em histórico local; nenhuma ação automática sem consentimento.

## UC-06 · Relatório profissional (assistência técnica)

- **Ator:** Técnico de TI (Studio).
- **Objetivo:** entregar laudo ao cliente.
- **Fluxo:** roda diagnóstico+benchmark → "Gerar Relatório" → PDF white-label com score, gargalos, antes/depois → entrega ao cliente.
- **Garantias:** dados do cliente ficam locais; export controlado.

## UC-07 · Monitoramento contínuo

- **Ator:** Criador de conteúdo.
- **Objetivo:** vigiar temperatura/throttling durante render.
- **Fluxo:** Monitoramento em tempo real → alerta se temp/throttle ultrapassa limite → sugere ação.
- **Garantias:** baixo overhead; alertas configuráveis.

## UC-08 · Limpeza segura

- **Ator:** Usuário comum.
- **Objetivo:** liberar espaço sem perder nada importante.
- **Fluxo:** Diagnóstico → "Limpeza" → preview do que será movido → confirma → arquivos vão para **quarentena** (não deletados) → recuperáveis por TTL.
- **Garantias:** nada apagado de imediato; recuperável.

## Matriz Caso × Módulo

| UC | Core | Monitor | Analyzer | Optimizer | GameBoost | Benchmark | Rollback | Report |
|----|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| 01 | ● | ● | ● | | | | | |
| 02 | ● | ● | ● | ● | | ● | ● | |
| 03 | ● | | | | | | ● | |
| 04 | ● | ● | | ● | ● | | ● | |
| 05 | ● | ● | ● | | | | | |
| 06 | ● | ● | ● | | | ● | | ● |
| 07 | ● | ● | ● | | | | | |
| 08 | ● | | ● | ● | | | ● | |
