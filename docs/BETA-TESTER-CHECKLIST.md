# TkSpeed 0.1.0-rc1 — Checklist para Beta Testers

**Versão:** 0.1.0-rc1 · **Data:** 2026-06-15
**Tempo estimado:** 45–60 minutos

Para cada item: marque ✅ (funciona), ❌ (quebrado), ou ⚠️ (funciona com ressalva).
Anote o que aconteceu em caso de ❌ ou ⚠️.

---

## 1. Instalação

- [ ] Installer NSIS executa sem erro
- [ ] App abre no Menu Iniciar após instalação
- [ ] Ícone aparece na taskbar com tamanho adequado (semelhante a Discord/VS Code)
- [ ] Splash screen aparece ao abrir (símbolo animado + barra de progresso)
- [ ] App carrega na Central de Comando em menos de 5 segundos

---

## 2. Central de Comando

**Fluxo feliz:**
- [ ] Clicar "Analisar" executa diagnóstico e exibe score (0–1000)
- [ ] CPU, RAM, SSD, Temperatura e Pontuação exibidos nos vitais
- [ ] "Estado da Máquina" mostra classificação (Excelente / Bom / Atenção / Crítico)
- [ ] Seção "Gargalos Ativos" mostra barras de CPU / RAM / GPU

**Consultor Inteligente:**
- [ ] Após análise, recomendações aparecem (pode demorar alguns segundos)
- [ ] Cada recomendação tem título, descrição, badge de risco e razão
- [ ] Botão de ação presente nas recomendações de Perfil

**Auto Pilot:**
- [ ] Botão "Otimizar Agora" aparece na barra superior
- [ ] Clicar abre modal com análise "Analisando o sistema…"
- [ ] Plano aparece com lista de otimizações seguras
- [ ] Botão "Iniciar X otimizações" clicável
- [ ] Durante execução: checkmarks aparecem conforme progresso
- [ ] Ao finalizar: mensagem de resultado exibida
- [ ] Botão "Otimizar Agora" fica desativado durante execução (não pode clicar duas vezes)

---

## 3. Laboratório de Performance

- [ ] Seção "Hardware ao Vivo" exibe RAM e CPU atual
- [ ] Clicar "CPU" em "Executar Benchmark" inicia benchmark (~15–30s)
- [ ] Resultado aparece na seção de sessões ao final
- [ ] Clicar "Detectar agora" em "Detector de Gargalo" retorna análise
- [ ] "Capturar FPS" funciona ou exibe mensagem clara sobre PresentMon

---

## 4. Central de Otimizações

- [ ] Lista de otimizações carrega com nomes e badges de risco (Seguro / Moderado / Avançado)
- [ ] Clicar "Aplicar e medir" inicia otimização (~30s) e exibe resultado
- [ ] Resultado mostra nome, decisão (Mantido/Revertido/Inconclusivo) e confiança
- [ ] Histórico de otimizações exibe entradas anteriores
- [ ] Card "Gerenciador de Inicialização" → botão "Abrir" navega para /startup

---

## 5. Gerenciador de Inicialização

- [ ] Tela carrega e exibe lista de apps de inicialização
- [ ] Apps agrupados por impacto (Alto / Médio / Baixo)
- [ ] Economia estimada visível por item
- [ ] Botão "Desabilitar" presente em itens HKCU
- [ ] Modal de confirmação aparece ao clicar "Desabilitar"
- [ ] Após confirmar: toast "desabilitado · snapshot #X criado" aparece
- [ ] Item aparece como desabilitado após ação
- [ ] Itens HKLM mostram "Requer admin" (desativado)
- [ ] Subtítulo da tela NÃO mostra "0 segundos" na carga inicial

---

## 6. Resultados

- [ ] Tela exibe histórico de otimizações aplicadas
- [ ] Sumário geral mostra: total testadas, mantidas, com ganho, ganho médio
- [ ] Cards com otimizações têm badge de decisão (Mantido/Revertido)
- [ ] Clicar "Ver comparação" expande os dados Antes × Depois
- [ ] Destaque hero mostra ANTES e DEPOIS com delta percentual colorido
- [ ] Tabela de métricas secundárias visível após expansão
- [ ] Botão "Ir para Otimizações" funciona no estado vazio

---

## 7. Central de Restauração

- [ ] Tela carrega sem erro (mesmo sem snapshots)
- [ ] Após otimização aplicada: snapshot aparece na lista
- [ ] Botão "Reverter" presente nos snapshots ativos
- [ ] Confirmação solicitada antes de reverter
- [ ] Após reverter: estado refletido na lista

---

## 8. Histórico

- [ ] Tela carrega sem erro (estado vazio correto se sem dados)
- [ ] Após análise: score aparece na timeline
- [ ] Após benchmark: sessão aparece na lista de benchmarks
- [ ] Dados persistem ao fechar e reabrir o app

---

## 9. Navegação e Interface

- [ ] Menu lateral: todos os itens clicáveis navegam para a tela correta
- [ ] Items "Em breve" (Jogos, Memória, Pontos, Relatórios, Configurações) mostram tela adequada — SEM mencionar "V3" ou jargão interno
- [ ] BrandHeader (logo + tagline) visível no topo do menu
- [ ] Versão `v0.1.0-rc1` visível no rodapé do menu
- [ ] Splash screen não duplica logo

---

## 10. Estabilidade e Dados

- [ ] App não crasha durante uso de 20+ minutos
- [ ] Fechar e reabrir mantém histórico e snapshots
- [ ] Executar análise múltiplas vezes não duplica entradas
- [ ] App responde após PC ficar em sleep/hibernate

---

## Informações a reportar

Para cada ❌ encontrado, inclua:

```
Tela: [nome da tela]
Item do checklist: [número e texto]
O que aconteceu: [descrição]
O que era esperado: [comportamento esperado]
Reproduzível: [sim / às vezes / uma vez]
Screenshot: [anexar se possível]
```

**Hardware do testador (preencher):**
- CPU: _______________
- RAM: ___ GB
- Storage: SSD / HDD / NVMe
- GPU: _______________
- Windows: 10 / 11 · Versão: _______________
- Antivírus: _______________
