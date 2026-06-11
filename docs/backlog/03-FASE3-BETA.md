# Fase 3 — Beta / v1.0 Comercial

**Meta:** product-fit + monetização. Perfis, licenciamento e atualizador para virar produto comercial.
**Critério de saída:** GA com conversão Free→Pro ≥ 3%, churn < 5%, auditoria de segurança sem achados críticos.

Epics: E12 Perfis · E13 Licenciamento · E14 Atualizador.

---

## TK-E12 · Perfis de desempenho

### TK-F121 · Perfis de otimização

#### TK-S1211 · Perfis built-in + custom (Equilíbrio/Gamer/Silencioso/Workstation)
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** `optimization_profiles` (items_json); aplicar perfil = plano via saga; CRUD de perfil custom; preview agregado.
- **AC:** aplicar perfil executa o conjunto de tweaks com snapshot único reversível.
- **Dependências:** TK-S0812
- **DoD+:** reverter perfil restaura tudo.

#### TK-S1212 · Agendamento de perfis/manutenção
- **Prioridade:** P3 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** scheduler (ex.: limpeza semanal); integrar housekeeping; respeitar bateria.
- **AC:** tarefa agendada roda e registra em auditoria.
- **Dependências:** TK-S1211

---

## TK-E13 · Licenciamento
> Ver [03-LICENCIAMENTO](../business/03-LICENCIAMENTO.md). Offline-first.

### TK-F131 · Validação de licença

#### TK-S1311 · Token assinado (Ed25519) + validação local
- **Prioridade:** P0 · **Esforço:** L (8) · **Risco:** 🔴
- **Tasks:** pubkey embarcada; validar assinatura/validade/binding; `license` table; gating por `features[]`.
- **AC:** licença válida desbloqueia features offline; token forjado é rejeitado.
- **Dependências:** TK-S0121
- **DoD+:** testes de adulteração (assinatura inválida, expirado, clock retrocedido).

#### TK-S1312 · Ativação + device binding + grace period
- **Prioridade:** P0 · **Esforço:** L (8) · **Risco:** 🔴
- **Tasks:** fluxo de ativação; device_id (fingerprint estável); limite de dispositivos; grace period pós-expiração; detecção de retrocesso de relógio.
- **AC:** ativa, vincula device, respeita limite; funciona offline; degrada p/ Free após grace.
- **Dependências:** TK-S1311

#### TK-S1313 · Backend de licenciamento (stateless) + faturamento
- **Prioridade:** P1 · **Esforço:** XL (13) · **Risco:** 🔴
- **Tasks:** emissão/renovação/revogação de token; webhooks Stripe + PIX/boleto; CRL leve; portal de gestão de device.
- **AC:** compra emite token; cancelamento revoga; renovação automática quando online.
- **Dependências:** TK-S1311
- **DoD+:** PCI/handling de pagamento revisado; sem segredo no cliente.

### TK-F132 · Gating de UI por plano

#### TK-S1321 · Feature flags por tier + telas de upgrade
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** flags por `features[]`; UI mostra recursos bloqueados + CTA upgrade; trial 14 dias.
- **AC:** recurso Pro aparece bloqueado no Free com caminho de upgrade; trial funciona.
- **Dependências:** TK-S1311

---

## TK-E14 · Atualizador

### TK-F141 · Auto-update assinado

#### TK-S1411 · Tauri Updater + assinatura
- **Prioridade:** P0 · **Esforço:** M (5) · **Risco:** 🔴
- **Tasks:** endpoint de releases; verificação de assinatura do update; rollback de update falho; canal stable/beta.
- **AC:** update só instala se assinatura válida; falha não corrompe instalação.
- **Dependências:** TK-S0141
- **DoD+:** teste de update malicioso/assinatura inválida rejeitado.

#### TK-S1412 · Instalador MSI/NSIS + bootstrap WebView2
- **Prioridade:** P1 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** WiX/NSIS; instalar WebView2 se ausente; assinatura Authenticode; install perMachine.
- **AC:** instala limpo em Win10 22H2+/Win11; binário assinado.
- **Dependências:** TK-S1411
