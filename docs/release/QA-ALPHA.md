# TkSpeed Alpha — QA (A7)

Plano de QA para transformar o build `0.1.0` (commit `74e84a4`) em um Alpha
utilizável por pessoas reais. Três etapas: validação local (gate do dev) →
matriz multi-máquina → kit do tester. Fecha no Go/No-Go.

Artefatos relacionados: [TEST-MATRIX.md](TEST-MATRIX.md) (roteiro por máquina),
[../../release/FEEDBACK.md](../../release/FEEDBACK.md), [../../release/LEIA-ME.txt](../../release/LEIA-ME.txt).

---

## A7.1 — Validação local (gate do desenvolvedor)

Rodar na máquina de build **antes** de enviar a qualquer tester. Cobre os dois
instaladores. Marque cada item; qualquer ✗ é bloqueio até resolver.

### Instalação
- [ ] **NSIS**: executar `TkSpeed_0.1.0_x64-setup.exe`; passar pelo SmartScreen ("Mais informações → Executar assim mesmo"); UAC aceito; instala sem erro; cria atalho.
- [ ] **SHA256**: `Get-FileHash` do `-setup.exe` confere com `SHA256SUMS.txt`.
- [ ] **MSI**: em outra limpeza (ou VM), instalar `TkSpeed_0.1.0_x64_en-US.msi`; instala sem erro.
- [ ] **WebView2**: em Windows 10 sem WebView2, o instalador o provê (não falha por falta dele).

### Abertura e telemetria
- [ ] App abre em < 2s; Dashboard carrega; hardware do cabeçalho está correto.
- [ ] CPU/RAM/Disco atualizam (~1s) e batem com o Gerenciador de Tarefas.
- [ ] Controles de janela (minimizar/maximizar/fechar) funcionam (janela sem borda).

### Funcional (caminho de valor)
- [ ] **Analisar**: gera TkSpeed Score + lista de gargalos.
- [ ] **Benchmark**: CPU, RAM e Storage rodam e exibem confiança.
- [ ] **Hardware ao vivo**: GPU NVIDIA com uso/VRAM/temp, ou "indisponível" honesto.
- [ ] **Otimização** (Plano de Alto Desempenho): mede antes/depois e decide manter/reverter.
- [ ] **Otimização** (Limpeza de Temporários): "Liberados X MB"; reversível.

### Segurança / reversibilidade (o pilar)
- [ ] **Rollback Center**: lista snapshots + otimizações; evidência expande.
- [ ] **Reverter otimização**: modal confirma → toast → estado restaurado.
- [ ] **Restaurar snapshot**: modal confirma → toast → estado restaurado.
- [ ] **Startup**: desabilitar item HKCU cria snapshot; reabilitar via Rollback Center restaura; item HKLM mostra "Requer admin" (desabilitado).
- [ ] **Autoteste de proteção**: os 4 passos PASSAM.

### Persistência, diagnóstico e remoção
- [ ] Fechar e reabrir: histórico/sessões/score persistem.
- [ ] **History**: página mostra score ao longo do tempo + sessões de benchmark.
- [ ] **Diagnóstico**: `Export-TkSpeedDiagnostic.ps1` gera `TkSpeed-Diagnostic.zip` (logs + db + system.txt) sem erro de parse.
- [ ] **Desinstalar** (NSIS e MSI): some de Apps; sem erro; dados em `%APPDATA%\TkSpeed` permanecem (documentado).

---

## A7.2 — Matriz multi-máquina (foco atribuído por perfil)

Meta mínima: ≥1 máquina por dimensão exigida (Intel, AMD, RTX, GTX, RX, iGPU,
Notebook, Desktop, Win10, Win11). Cada perfil roda o **roteiro completo** de
[TEST-MATRIX.md](TEST-MATRIX.md); a coluna "Foco" diz o que validar com afinco
naquele perfil (onde ele é o melhor — ou pior — caso).

| ID | CPU | GPU | Forma | SO | Foco específico (o que esse perfil prova) |
|----|-----|-----|-------|----|-------------------------------------------|
| **M1** | Intel | NVIDIA **RTX** | Notebook | **Win11** | GPU NVML completa (uso/VRAM/clock/temp); **FPS via PresentMon** (com admin); térmico → "inconclusivo" honesto no 2º bench. |
| **M2** | **AMD** | **AMD RX** | Desktop | Win11 | **GPU "indisponível" honesta** (NVML é NVIDIA-only) sem crash; Plano de Alto Desempenho; benches em CPU AMD. |
| **M3** | Intel | NVIDIA **GTX** | Desktop | **Win10** (22H2) | **SmartScreen no Win10**; **WebView2 auto-instalado**; GTX antiga no NVML; SSD SATA. |
| **M4** | AMD | **iGPU** (Radeon) | Notebook | Win10 | **iGPU → "indisponível"**; **HDD ou SATA lento** (Storage bench lento = normal; detector indica HDD); pior caso de hardware. |
| **M5** | Intel | iGPU (Intel) | Notebook | Win11 | Sem GPU dedicada; **sem admin** (caminho padrão): cleanup WU libera pouco/zero sem travar; PresentMon ausente → erro claro, não crash. |

> Cobertura conferida: Intel (M1,M3,M5) · AMD (M2,M4) · RTX (M1) · GTX (M3) · RX (M2) · iGPU (M4,M5) · Notebook (M1,M4,M5) · Desktop (M2,M3) · Win10 (M3,M4) · Win11 (M1,M2,M5).
> Se faltar acesso a algum perfil, **registrar como gap** (não marcar como coberto).

### Casos de atenção (anotar comportamento, não são bugs)
- Sem NVIDIA → GPU "indisponível" (M2/M4/M5).
- Notebook em carga → otimização pode dar "inconclusivo" (térmico) — honesto.
- WU cache sem admin → libera pouco; não trava.
- PresentMon ausente → "Capturar FPS" dá erro claro.

---

## A7.3 — Alpha fechado (kit do tester)

### O que o tester recebe (pacote `TkSpeed-0.1.0-alpha-x64.zip`)
- `TkSpeed_0.1.0_x64-setup.exe` (recomendado) ou `.msi`.
- `LEIA-ME.txt` (instalar, SmartScreen, integridade, desinstalar).
- `Export-TkSpeedDiagnostic.ps1`, `FEEDBACK.md`, `CHANGELOG.md`, `LICENSE-ALPHA.txt`, `SHA256SUMS.txt`.

### Checklist do tester (versão curta — detalhe em TEST-MATRIX.md)
1. [ ] Conferir SHA256 do instalador (opcional).
2. [ ] Instalar e abrir.
3. [ ] Preencher a **linha da sua máquina** na matriz de cobertura.
4. [ ] Rodar o **roteiro funcional** (15 passos de TEST-MATRIX.md), marcando OK/✗.
5. [ ] Usar pelo menos uma otimização e **reverter** (testar o rollback de verdade).
6. [ ] Se algo der errado: anotar passo a passo e mensagem de erro.
7. [ ] Gerar diagnóstico: clicar direito em `Export-TkSpeedDiagnostic.ps1` → "Executar com PowerShell" → gera `TkSpeed-Diagnostic.zip`.
8. [ ] Preencher `FEEDBACK.md`.
9. [ ] Enviar **`TkSpeed-Diagnostic.zip` + `FEEDBACK.md`** ao time.

### Coleta (o que volta de cada tester)
- **Logs + banco + sistema**: dentro de `TkSpeed-Diagnostic.zip` (`logs/`, `tkspeed.db`, `system.txt`). Sem dados pessoais.
- **Percepção + bugs**: `FEEDBACK.md`.
- Distribuição sugerida: GitHub Releases (técnicos) ou link direto do zip (casuais).

---

## Gate Go / No-Go (Alpha externo)

**GO** somente se TODOS forem verdade:
- [ ] A7.1 (validação local) 100% verde na máquina de build, **NSIS e MSI**.
- [ ] CI verde no último commit (build + test + clippy + frontend).
- [ ] **Rollback comprovado** (reverter otimização + restaurar snapshot) em ≥2 máquinas reais distintas.
- [ ] Degradação graciosa **sem NVIDIA** confirmada em ≥1 máquina (AMD/iGPU) — sem crash.
- [ ] Exportação de diagnóstico funciona em ≥2 máquinas.
- [ ] **0 defeitos P0** abertos (perda de dado, falha de rollback, crash no boot, instalador quebrado).
- [ ] P1 triados (com workaround ou aceitos para o próximo Alpha).

**NO-GO** se qualquer P0 estiver aberto, ou se o rollback falhar em qualquer máquina
(o pilar do produto é a reversibilidade — falha aqui é bloqueio absoluto).

> Severidades: **P0** = perda de dado / rollback falho / crash no boot / instalador quebrado.
> **P1** = funcionalidade importante quebrada com workaround. **P2** = cosmético/menor.
> Defeitos viram issues no Git (`fix/<assunto>`), referenciados no `CHANGELOG.md`.
