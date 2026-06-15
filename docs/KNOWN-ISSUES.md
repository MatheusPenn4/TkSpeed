# TkSpeed 0.1.0-rc1 — Problemas Conhecidos

**Atualizado:** 2026-06-15 · **Build:** 0.1.0-rc1

Este documento lista todos os problemas conhecidos no momento da distribuição beta.  
Prioridade: **P0** = bloqueante · **P1** = impacto visível · **P2** = cosmético / baixo impacto.

---

## Sem bloqueantes (P0)

Nenhum problema P0 identificado nesta versão.

---

## P1 — Impacto visível, não bloqueante

### KI-001 · Benchmark de Storage pode demorar 60s+ em HDDs
**Tela:** Laboratório de Performance  
**Reprodução:** Executar benchmark "Storage" em HDD mecânico (não-SSD).  
**Comportamento:** O benchmark leva mais tempo do que os ~30s esperados para SSDs.  
**Workaround:** Aguardar até 90 segundos. O resultado aparece normalmente ao final.  
**Status:** Em análise para a próxima versão.

### KI-002 · Auto Pilot não exibe estimativa de tempo restante
**Tela:** Central de Comando → Modal "Otimizar Agora"  
**Reprodução:** Iniciar o Auto Pilot com múltiplas otimizações na fila.  
**Comportamento:** A barra de progresso mostra checkmarks mas sem estimativa de tempo.  
**Impacto:** Usuário não sabe quanto tempo falta para a execução terminar.  
**Status:** Melhoria planejada para 0.1.1.

### KI-003 · Captura de FPS requer PresentMon externo
**Tela:** Laboratório de Performance → Captura de Jogo  
**Reprodução:** Clicar "Capturar FPS" sem ter o PresentMon instalado.  
**Comportamento:** O botão fica desabilitado ou exibe erro silencioso.  
**Workaround:** Usar o botão "Demo sem jogo" para validar o pipeline de FPS sem um jogo aberto. Para captura real: baixar `PresentMon-x64.exe` e colocar em `%APPDATA%\TkSpeed\tools`.  
**Status:** Documentado. Integração automática planejada para versão futura.

---

## P2 — Cosmético / baixo impacto

### KI-004 · Menu lateral não colapsa em janelas estreitas (< 480px)
**Descrição:** Em janelas muito estreitas, o menu lateral não colapsa e pode sobrepor o conteúdo.  
**Impacto:** Não afeta uso normal em desktop. Não relevante para a distribuição beta atual.

### KI-005 · Ícone duplicado para "Relatórios" e "Resultados" no menu
**Descrição:** Ambos os itens do menu usam o mesmo ícone de relatórios.  
**Impacto:** Estético apenas. Navegação funciona corretamente.

### KI-006 · Detector de Gargalo requer carga real para resultado preciso
**Tela:** Laboratório de Performance / Central de Comando  
**Descrição:** O detector de gargalo amostra 2 segundos de uso do sistema. Em desktop idle, o resultado indica "Balanceado" mesmo que haja gargalos latentes.  
**Workaround:** Clicar "Detectar agora" enquanto um jogo ou aplicativo pesado está aberto.

### KI-007 · GPU indisponível em sistemas sem NVIDIA/NVML
**Tela:** Laboratório de Performance → Hardware ao vivo  
**Descrição:** O painel de GPU exibe "Indisponível" em sistemas com GPUs AMD ou Intel integradas.  
**Impacto:** Monitoramento de GPU limitado. CPU, RAM e Storage funcionam normalmente em todos os sistemas.  
**Status:** Suporte AMD planejado para versão futura.

### KI-008 · Score inicial pode demorar para aparecer após primeira análise
**Tela:** Central de Comando  
**Descrição:** Na primeira análise após a instalação, o TkSpeed Score pode levar 3–5 segundos extras para ser calculado, pois o benchmark inicial precisa ser coletado.  
**Workaround:** Aguardar. O score aparece sem necessidade de ação.

---

## Comportamentos esperados (não são bugs)

| Comportamento | Explicação |
|---|---|
| "Requer admin" em alguns apps de inicialização | Apps do sistema (HKLM) exigem privilégio de administrador para serem desabilitados. Isso é por design de segurança. |
| Otimização "Revertida" após aplicação | O sistema mediu e não detectou ganho real. O estado anterior foi restaurado automaticamente. Isso é o comportamento correto. |
| Benchmark CPU leva ~15–30s | O benchmark executa múltiplas passagens para garantir confiança estatística. |
| Recomendações do Consultor variam entre sessões | O Consultor aprende com os benchmarks do seu sistema. Mais benchmarks = recomendações mais precisas. |

---

## Como reportar novos problemas

Para reportar um bug não listado aqui, inclua:
1. Tela onde ocorreu
2. Passos para reproduzir
3. O que aconteceu vs. o que era esperado
4. Screenshot (se possível)
5. Hardware do sistema (CPU, RAM, GPU, tipo de storage)
