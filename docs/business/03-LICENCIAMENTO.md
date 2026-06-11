# 03 · Arquitetura de Licenciamento

> Projetada desde o início para suportar 4 planos sem refatoração. Princípio: **offline-first** — o produto nunca trava o usuário pago por estar sem internet.

## 1. Planos e funcionalidades

| Capacidade | Free | Pro | Studio | Enterprise |
|---|:--:|:--:|:--:|:--:|
| Monitoramento tempo real | básico | completo | completo | completo |
| Diagnóstico / gargalos | limitado | completo | completo | completo |
| Otimização | tweaks `safe` | catálogo completo + perfis | completo | completo + políticas |
| Rollback | ✅ | ✅ histórico estendido | ✅ | ✅ + auditoria central |
| Benchmark + antes/depois | 1 suite | todas | todas | todas |
| Game Boost | 1 perfil | ilimitado | ilimitado | ilimitado |
| Digital Twin (retenção) | 7 dias | 1 ano | ilimitado | ilimitado |
| Relatórios | HTML básico | PDF/HTML premium | **white-label** | white-label + agregado |
| TkAI (Fase 4) | — | ✅ | ✅ | ✅ |
| Multi-dispositivo (mesma conta) | 1 | até 3 | até 5 | conforme contrato |
| Fleet / console central | — | — | — | ✅ |
| Deploy via GPO/MDM/MSI silencioso | — | — | — | ✅ |
| SSO / SAML | — | — | — | ✅ |
| Suporte | comunidade | prioritário | dedicado | SLA + gerente de conta |
| Preço alvo | R$ 0 | ~R$ 24,90/mês · R$ 199/ano | ~R$ 79,90/mês · R$ 699/ano | sob consulta (por seat/ano) |

## 2. Modelo de licença

- **Free:** sem chave; recursos gated por flag local (verificável offline).
- **Pro/Studio:** assinatura → **chave de licença assinada** (JWT/token com assinatura assimétrica Ed25519).
- **Enterprise:** licença por volume (seats) + ativação por console/arquivo de licença, deploy silencioso.

A licença é um **token assinado pelo servidor** contendo: `tier`, `seats`, `expires_at`, `device_binding`, `features[]`. O cliente valida a **assinatura** com a chave pública embarcada — não precisa do servidor para confiar no conteúdo.

## 3. Mecanismo de ativação

```
1. Usuário compra → backend emite token assinado (Ed25519).
2. App recebe token, valida assinatura com pubkey embarcada.
3. App vincula ao device (device_id = hash estável de fingerprint de hardware).
4. Token + estado salvo localmente (license table, ver docs/04-BANCO-DADOS.md).
```

- **Device binding** evita compartilhamento ilimitado (limite de N dispositivos por conta).
- Reativação/transferência de device via portal (libera slot).

## 4. Sistema offline

- Após ativada, a licença funciona **100% offline** até `expires_at`.
- Validação local = checar assinatura + validade + binding. **Sem internet necessária** no uso diário.
- **Grace period:** mesmo após o vencimento nominal, há janela (ex.: 14 dias) antes de degradar para Free — protege contra falha de renovação/conectividade.

## 5. Sistema online

- Verificação periódica **leve e opcional** (ex.: a cada 7 dias) para:
  - renovar token (assinatura curta, ex. 30 dias, auto-renovada quando online);
  - revogar licença comprometida (lista de revogação);
  - sincronizar troca de plano.
- Servidor de ativação **stateless** atrás de CDN → escala horizontal trivial.

## 6. Validação de licença (camadas)

1. **Assinatura criptográfica** (Ed25519) — impede forjar token.
2. **Validade temporal** (`expires_at` + grace).
3. **Device binding** (limite de dispositivos).
4. **Revogação** (CRL leve quando online).
5. **Gating de feature** por `features[]` do token (não por versão de build).

## 7. Prevenção contra fraude

| Vetor | Mitigação |
|---|---|
| Forjar licença | Assinatura assimétrica; pubkey embarcada; cliente nunca emite token |
| Compartilhar chave | Device binding + limite de dispositivos + detecção de ativações anômalas |
| Adulterar relógio (burlar expiração) | Carimbo do servidor na última sincronização; detecção de retrocesso de clock |
| Patch do binário (remover checagem) | Assinatura Authenticode; ofuscação de pontos críticos; checagens redundantes; o valor real está em features server-gated (TkAI/cloud) |
| Revenda de Enterprise | Watermark de licença + contrato + telemetria de seats agregada |
| Keygen | Sem algoritmo de geração no cliente; tokens vêm só do backend |

> **Filosofia anti-pirataria pragmática:** não gastar fortuna em DRM agressivo (prejudica clientes legítimos). Tornar a fraude **inconveniente**, e ancorar valor em serviços que exigem servidor (TkAI cloud, sync, marketplace, suporte) — difíceis de piratear.

## 8. Faturamento

- Stripe (internacional) + provedor BR (PIX/boleto/cartão).
- Webhooks de pagamento → emissão/renovação/revogação de token.
- Trials (ex.: 14 dias Pro) sem cartão para reduzir atrito.
