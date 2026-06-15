# TkSpeed 0.1.0 — Release Candidate 1

**Data:** 15 de junho de 2026
**Build:** `0.1.0-rc1`
**Plataforma:** Windows 10/11 x64
**Status:** Release Candidate — aprovado para distribuição beta

---

## O que é o TkSpeed?

TkSpeed é um otimizador de performance para Windows focado em gamers. Diferente de otimizadores tradicionais que aplicam mudanças às cegas, o TkSpeed **mede antes e depois de cada otimização** e só mantém o que comprova ganho real. Tudo é reversível com um clique.

---

## Novidades nesta versão

### ⚡ Otimizar Agora — Auto Pilot

Um clique. O sistema analisa sua máquina, seleciona as otimizações mais seguras e aplica automaticamente — com progresso em tempo real. Cada otimização é medida. Se não houver ganho, é revertida.

### 📊 Centro de Resultados

Pela primeira vez, você vê exatamente o que mudou: score antes, score depois, ganho percentual por métrica. Não é estimativa — são seus dados reais, medidos no seu PC.

### ⚡ Gerenciador de Inicialização

Veja todos os apps que estão atrasando o boot do Windows. Desabilite os de alto impacto com um clique. Cada alteração cria um ponto de restauração automático — você pode reverter a qualquer momento.

---

## O que está funcional nesta versão

| Funcionalidade | Status |
|---|---|
| Central de Comando + diagnóstico | ✅ Completo |
| TkSpeed Score (0–1000) | ✅ Completo |
| Consultor Inteligente (top-5 recomendações) | ✅ Completo |
| Auto Pilot — Otimizar Agora | ✅ Novo |
| 9 otimizações mensuráveis | ✅ Completo |
| 4 perfis de performance | ✅ Completo |
| Benchmark CPU / RAM / Storage / FPS | ✅ Completo |
| Detector de gargalo | ✅ Completo |
| Centro de Resultados (Antes × Depois) | ✅ Novo |
| Gerenciador de Inicialização | ✅ Novo |
| Rollback / Restauração | ✅ Completo |
| Histórico de benchmarks e score | ✅ Completo |

### Em breve (próximas versões)

- Game Center — detecção de jogos + perfil 1-clique + FPS antes/depois
- Memory Manager — flush de RAM standby + monitor ao vivo
- Relatórios exportáveis (PDF/imagem)
- Configurações

---

## Requisitos

- Windows 10 22H2 ou superior / Windows 11
- x64 (AMD64)
- 4 GB RAM mínimo
- 100 MB de espaço em disco
- .NET Runtime não necessário — app portátil (Tauri/WebView2)
- WebView2 Runtime (incluído no Windows 11; instalado automaticamente no Windows 10)

---

## Instalação

**NSIS (recomendado):** `TkSpeed_0.1.0_x64-setup.exe` — instala com atalho no Menu Iniciar e suporte a desinstalar
**MSI:** `TkSpeed_0.1.0_x64_en-US.msi` — para ambientes corporativos ou deploy silencioso

Tamanhos:
- NSIS installer: ~4.1 MB
- MSI: ~5.9 MB

---

## Notas de segurança

- Todas as otimizações são **reversíveis** — o app cria um snapshot antes de qualquer mudança
- Otimizações de nível HKLM (sistema) exigem execução como administrador
- Nenhum dado é enviado para servidores externos — tudo local (SQLite em `%APPDATA%\TkSpeed`)
- FPS Capture via PresentMon requer execução como administrador (opcional)

---

## Bugs conhecidos (P2 — não bloqueantes)

- Nav lateral não colapsa em janelas muito estreitas (< 480px) — não afeta uso normal em desktop
- "Relatórios EM BREVE" e "Resultados" usam o mesmo ícone no menu lateral
- Classificação interna "Critico" sem acento no backend (display correto na UI)

---

## Problemas? Feedback?

Esta é uma versão Release Candidate para beta testers selecionados. Reporte problemas diretamente ao time de produto com:
1. Descrição do problema
2. Tela onde ocorreu
3. Screenshot se possível
4. Log de diagnóstico exportado (Central de Comando → menu → Exportar diagnóstico)
