# TkSpeed — Plano de Distribuição Alpha (0.1.0)

> Runbook de release. Nenhuma funcionalidade nova — apenas empacotar, instalar e
> distribuir o que já existe + tooling externo de suporte (script de diagnóstico,
> formulário de feedback, matriz de testes).

---

## FASE 1 · Geração do Build (release)

**Pré-requisitos na máquina de build (a sua):**
- Rust toolchain `stable-msvc` + Visual Studio Build Tools (workload "Desktop C++") — já instalados.
- Node + npm — já instalados.
- Internet (na 1ª vez o Tauri baixa o NSIS e o WiX para gerar os instaladores).

**Comando único (gera frontend + binário release + instaladores):**
```powershell
cd C:\Users\User\Documents\TkSpeed
npx '@tauri-apps/cli@^2' build
```
Isso executa, em ordem: `beforeBuildCommand` (build do frontend → `apps/desktop/dist`),
`cargo build --release` e o empacotamento (NSIS + MSI).

**Artefatos gerados** — num workspace Cargo, a pasta `target/` fica na **raiz** (`TkSpeed\target\`, NÃO em `src-tauri\`):
| Artefato | Caminho |
|---|---|
| Executável "cru" | `target\release\tkspeed.exe` |
| **Instalador NSIS (recomendado)** | `target\release\bundle\nsis\TkSpeed_0.1.0_x64-setup.exe` |
| Instalador MSI | `target\release\bundle\msi\TkSpeed_0.1.0_x64_en-US.msi` |

> O `tkspeed.exe` cru roda, mas **distribua o instalador** (cuida de atalhos, ícone,
> registro de desinstalação e bootstrap do WebView2). Não copie só o .exe.

**Dependência no PC do usuário:** WebView2 Runtime (já presente no Windows 11; no Windows 10
o instalador NSIS faz o bootstrap automaticamente).

---

## FASE 2 · Instalador — escolha: **NSIS (.exe)**

**Por quê NSIS e não MSI:** mais amigável para amigos/usuários comuns (assistente simples, um
único `.exe`), faz bootstrap do WebView2 e gera desinstalador. MSI é melhor para implantação
corporativa (GPO/Intune) — mantemos como alternativa, mas o **canal Alpha usa o `-setup.exe`**.

**Metadados já configurados** (`src-tauri/tauri.conf.json`):
- Nome do produto: **TkSpeed**
- Versão: **0.1.0**
- Identifier: `com.tkspeed.app`
- Publisher: **TkSpeed** · Copyright: © 2026 TkSpeed — Alpha
- Ícone: `src-tauri/icons/icon.ico` (gerado)
- Idiomas do instalador: PT-BR + EN · Modo: por máquina (pede UAC uma vez)

> ⚠️ **Assinatura de código:** o instalador é **não assinado** (Alpha). O Windows mostrará
> SmartScreen ("Windows protegeu seu PC"). Oriente o testador: **Mais informações → Executar
> assim mesmo**. Assinatura (certificado) fica para o Beta. Está no LEIA-ME.

---

## FASE 3 · Empacotamento — "TkSpeed Alpha 0.1.0"

Monte uma pasta/zip para enviar:
```
TkSpeed Alpha 0.1.0/
├── TkSpeed_0.1.0_x64-setup.exe     (copiado de target\release\bundle\nsis\)
├── LEIA-ME.txt                      (release/LEIA-ME.txt)
├── CHANGELOG.md                     (release/CHANGELOG.md)
├── LICENSE-ALPHA.txt                (release/LICENSE-ALPHA.txt)
├── FEEDBACK.md                      (release/FEEDBACK.md  — o testador preenche e devolve)
└── Export-TkSpeedDiagnostic.ps1     (release/Export-TkSpeedDiagnostic.ps1)
```
Comando para montar o pacote (após o build):
```powershell
$ver = "0.1.0"; $pkg = "$env:USERPROFILE\Desktop\TkSpeed Alpha $ver"
New-Item -ItemType Directory -Force $pkg | Out-Null
Copy-Item "target\release\bundle\nsis\TkSpeed_${ver}_x64-setup.exe" $pkg
Copy-Item release\LEIA-ME.txt, release\CHANGELOG.md, release\LICENSE-ALPHA.txt, release\FEEDBACK.md, release\Export-TkSpeedDiagnostic.ps1 $pkg
Compress-Archive -Path "$pkg\*" -DestinationPath "$pkg.zip" -Force
"Pacote: $pkg.zip"
```

---

## FASE 4 · Exportação de Diagnóstico (script externo — não altera o app)

`release/Export-TkSpeedDiagnostic.ps1` gera **`TkSpeed-Diagnostic.zip`** na Área de Trabalho
contendo: o banco `tkspeed.db` (hardware, score, findings, benchmarks, histórico de otimizações),
os **logs** (`%APPDATA%\TkSpeed\logs`) e um resumo de sistema (`system.txt`). O testador roda e
te envia o zip → você abre o `.db` (qualquer visualizador SQLite) + logs para **suporte remoto**.
Uso (no PC do testador): clique direito → "Executar com PowerShell" (ou `pwsh Export-TkSpeedDiagnostic.ps1`).

---

## FASE 5 · Feedback
`release/FEEDBACK.md` — formulário curto (score faz sentido? análise? otimização funcionou?
ganho perceptível? bugs?). O testador preenche e devolve junto com o `TkSpeed-Diagnostic.zip`.

---

## FASE 6 · Matriz de Testes
Ver [`TEST-MATRIX.md`](TEST-MATRIX.md) — checklist por configuração (Win10/11, Intel/AMD,
RTX/GTX/RX, SSD/HDD, notebook/desktop) + roteiro funcional por máquina.

---

## FASE 7 · Distribuição (5–20 pessoas)

| Opção | Vantagens | Desvantagens | Veredito |
|---|---|---|---|
| **GitHub Releases** | versionado, link direto, changelog, sem login p/ baixar (repo público) | repo **privado** exige login p/ baixar assets; expõe o nome do projeto | ✅ **melhor para testadores técnicos** / repo público |
| **OneDrive (link)** | já vem no Windows, link fácil, controla acesso | sem versionamento; alguns navegadores avisam de .exe | ✅ **melhor para amigos não-técnicos** |
| **Google Drive** | universal, link fácil | **Drive frequentemente bloqueia/avisa .exe**; pode pedir "baixar mesmo assim" | ⚠️ funciona, mas atritos com .exe |
| **Dropbox** | link direto, confiável | limite de banda no plano free; conta necessária p/ alguns recursos | ⚠️ ok, menos comum |

**Recomendação:** distribua o **`.zip`** (não o `.exe` solto — evita bloqueios e leva LEIA-ME/feedback junto).
- Repo no GitHub → **GitHub Releases** com o zip anexado + corpo = CHANGELOG (link estável, profissional).
- Sem repo / amigos casuais → **OneDrive** com link de compartilhamento.
- Sempre avise sobre o **SmartScreen** (instalador não assinado) no texto do convite.

---

## FASE 8 · Checklist final (antes de enviar)

**Build & instalador**
- [ ] `npx @tauri-apps/cli@^2 build` concluiu sem erro
- [ ] `TkSpeed_0.1.0_x64-setup.exe` existe em `bundle\nsis\`
- [ ] Instalado **numa máquina limpa** (VM/2º PC): instala, abre, desinstala sem deixar lixo

**Funcional (smoke test na sua máquina)**
- [ ] App abre; Dashboard mostra hardware + telemetria ao vivo
- [ ] "Analisar Agora" → score + findings
- [ ] Performance Lab → benchmark CPU/RAM/Storage + comparação com confiança
- [ ] Otimizações → aplicar Plano de Energia (bench antes/depois + decisão)
- [ ] Otimizações → Limpeza (libera espaço) e **Reverter** (restaura da quarentena)
- [ ] Snapshot/Rollback (Proteção) → autoteste PASSOU
- [ ] Controles de janela (min/max/fechar) funcionam
- [ ] Logs em `%APPDATA%\TkSpeed\logs`

**Pacote**
- [ ] `Export-TkSpeedDiagnostic.ps1` gera o zip corretamente
- [ ] LEIA-ME, CHANGELOG, LICENSE-ALPHA, FEEDBACK incluídos
- [ ] `.zip` do pacote montado e testado (extrai e instala)

---

## Diagnóstico remoto (como dar suporte)
1. Peça ao testador: rodar `Export-TkSpeedDiagnostic.ps1` → enviar `TkSpeed-Diagnostic.zip` + `FEEDBACK.md`.
2. Você: abrir `tkspeed.db` (ex.: DB Browser for SQLite) → tabelas `hardware_inventory`, `scores`,
   `findings`, `benchmark_sessions/metrics`, `optimization_runs`, `audit_log`.
3. Ler `logs\tkspeed.log.<data>` (JSON) para erros; `audit_log` para o que foi aplicado/revertido.
4. Cruzar com o `FEEDBACK.md` (percepção do usuário vs evidência medida).
