# Changelog — TkSpeed

Formato: [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/)

---

## [0.1.0-rc1] — 2026-06-15

### Produto — V4.4 Productization Phase

Esta versão transforma o TkSpeed de plataforma técnica em produto orientado a resultado percebido.

#### Adicionado
- **Centro de Resultados** (`/results`) — tela dedicada Antes × Depois com sumário de ganhos, destaque hero por métrica principal, tabela comparativa, delta percentual e status por otimização
- **Auto Pilot** — botão "Otimizar Agora" na Central de Comando; filtra recomendações seguras e aplica sequencialmente com progresso em tempo real e rollback automático se não houver ganho
- **Gerenciador de Inicialização** (`/startup`) — tela própria com lista de apps agrupados por impacto (Alto/Médio/Baixo), economia estimada de boot, toggle de desabilitar com snapshot reversível automático
- "Resultados" adicionado ao menu lateral (grupo Análise)
- "Inicialização" promovido de "Em breve" para tela funcional

#### Corrigido (RC1 Audit)
- **[P1]** Jargão interno "V3" removido das telas "Em breve" — substituído por "Esta funcionalidade estará disponível em breve."
- **[P1]** Startup Manager: subtítulo não exibe mais "0 segundos" antes dos dados carregarem
- **[P1]** Central de Otimizações: seção "Análise de Inicialização" duplicada removida — substituída por card com link direto para `/startup`
- **[P1]** Auto Pilot: botão desativado enquanto operação já está em curso (previne race condition)

---

## [0.1.0-alpha] — 2026-06-06

### Design System V3 — Apex × Quantum Mint

#### Adicionado
- Design system "Apex": tokens CSS, Geist Sans + Geist Mono, paleta Quantum Mint `#58F2D2` + Ion Blue `#39C7FF`, grafite `#08090C → #1A1E27`
- Splash screen redesenhada: símbolo único + 3 anéis animados + progresso de inicialização + wordmark em CSS
- Ícone do aplicativo regenerado: fundo transparente, símbolo TK colorido, ~90% da canvas (taskbar equivalente a Discord/VS Code)
- `BrandSymbol`, `BrandHeader`, `BrandWatermark` — componentes de marca isolados
- `AxRail`, `AxShell`, `AxCard`, `AxButton`, `AxBadge`, `AxMetric`, `AxModal`, `AxToast`, `AxEmptyState`, `AxEvidenceCard`, `AxSignalLockMeter`, `AxIcon`, `AxCommandPalette`
- Consultor Inteligente (`advisor_recommendations`) integrado na Central de Comando
- Comando `advisor_apply_profile` com medição antes/depois de perfil

---

## [0.1.0-alpha-3] — 2026-06-01 *(pré-Apex)*

### Fase 2 — Plataforma técnica completa

#### Adicionado
- Evidence-based optimization loop: snapshot → benchmark antes → aplica → benchmark depois → compara vs noise floor → verdict
- `RecommendationEngine`: top-N recomendações pontuadas por evidência
- `MeasurementPipeline`: before/after full benchmark
- Machine fingerprinting (hash de hardware) para evidência por máquina
- Noise floor automático (baseline estatístico por suite)
- Rollback Center com integridade criptográfica (SHA-256)
- History page: score timeline + sessões de benchmark
- Startup analysis + disable reversível com snapshot (HKCU)
- FPS capture via PresentMon + synthetic fallback
- Bottleneck detector (CPU/GPU/RAM/Thermal/Storage)
- System capabilities matrix
- 302 testes unitários e de integração

---

## [0.1.0-alpha-1] — 2026-05-15 *(MVP)*

#### Adicionado
- Diagnóstico de hardware: CPU, RAM, Storage, GPU (NVML), temperaturas
- TkSpeed Score (0–1000) com classificação: Elite → Crítico
- 9 otimizações do sistema mensuráveis
- 4 perfis de performance (Balanceado, Alto Desempenho, Ultra, Restrição)
- SQLite local com migrations automáticas
- Tauri 2.x bridge: 23 comandos registrados
- PresentMon integration para FPS real
