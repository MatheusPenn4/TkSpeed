# 08 · Plano de Segurança

> Segurança é **prioridade absoluta**. O produto altera o sistema operacional — qualquer falha pode danificar a máquina do usuário. Tudo é reversível, auditável e minimamente privilegiado.

## 1. Modelo de ameaças (STRIDE resumido)

| Ameaça | Vetor | Mitigação |
|---|---|---|
| Spoofing | Update malicioso / DLL hijack | Updates e binários assinados (Authenticode); verificação de assinatura no updater |
| Tampering | Alteração do DB / snapshots | Hash de integridade em snapshots; DB em `%APPDATA%` com ACL do usuário |
| Repudiation | "Não fui eu que mudei" | `audit_log` append-only com timestamp/ator |
| Information disclosure | Vazamento de telemetria | Telemetria **local** por padrão; envio só opt-in e anonimizado |
| DoS | Tweak que trava o SO | `SafetyGuard` + verificação pós-condição + rollback automático |
| Elevation of privilege | Abuso do componente elevado | Superfície elevada mínima, sem IPC arbitrário, validação de comando |

## 2. Princípio do menor privilégio

- O app roda **non-elevated** por padrão.
- Ações que exigem privilégio (Registry HKLM, serviços, planos de energia globais) passam pelo `PermissionBroker`, que dispara **UAC** apenas no momento exato e descreve a ação.
- Componente elevado (helper) executa um **conjunto fechado e validado** de operações — nunca comandos arbitrários vindos da UI.
- Sem `powershell -Command` com string concatenada de input do usuário (evita injeção). Operações usam APIs nativas (`tk-platform-win`) sempre que possível.

## 3. Segurança de otimizações

- Cada `Optimization` declara `risk_level` (`safe | moderate | advanced`).
- `safe`: aplica direto (com snapshot). `moderate`/`advanced`: exigem confirmação explícita com explicação de impacto.
- **Whitelist/Blacklist** de serviços e chaves: nunca toca em itens críticos do SO.
- `preview()` mostra exatamente o que mudará antes de aplicar.
- Verificação pós-aplicação; falha → rollback automático.

## 4. Integridade & supply chain

- Dependências auditadas (`cargo audit`, `cargo deny`, `pnpm audit`) no CI.
- SBOM gerado por release.
- Reprodutibilidade de build; binários assinados; releases com checksums publicados.
- Tauri capabilities mínimas (allowlist explícito de comandos/APIs).

## 5. Tauri hardening

- `capabilities/default.json` concede só os comandos necessários por janela.
- CSP estrita no WebView; sem `eval`, sem remote code.
- Sem `shell.open` arbitrário; navegação externa controlada.
- IPC tipado e validado; payloads sanitizados no boundary do `bridge`.

## 6. Dados & privacidade

- Telemetria, scores e histórico ficam **no dispositivo**.
- Nenhum dado pessoal sai sem consentimento explícito (LGPD/GDPR-friendly).
- Crash reports e analytics são **opt-in** e anonimizados.
- Exportações (relatórios) ficam sob controle do usuário.

## 7. Licenciamento seguro

- Ativação offline-first com chave assinada; verificação online opcional com *grace period*.
- Sem telemetria de uso atrelada a identidade.

## 8. Processo

- Threat modeling revisado por fase.
- **Auditoria de segurança externa antes do 1.0.**
- Programa de divulgação responsável (security.txt) pós-lançamento.
- Princípio: **fail-safe** — em dúvida, não alterar; preferir reverter.
