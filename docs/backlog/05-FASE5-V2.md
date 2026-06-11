# Fase 5 — v2.0 (TkAI, Cloud opcional, Enterprise)

**Meta:** inteligência, sincronização opt-in e B2B de alto valor.
**Critério de saída:** TkAI respondendo top-20 perguntas com evidência; 1 contrato Enterprise; sync opt-in estável.

Epics: E18 TkAI · E19 Cloud opcional · E20 Enterprise.

> Ver [06-TKAI-IA](../business/06-TKAI-IA.md) e [11-ESCALABILIDADE](../11-ESCALABILIDADE.md).

---

## TK-E18 · TkAI

### TK-F181 · Camada de inteligência local (L1+L2)

#### TK-S1811 · L1 — motor de explicação por regras/correlação
- **Prioridade:** P1 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:** respostas determinísticas sobre o Twin (gargalo/regressão/processo culpado) com citação de evidência; templates de explicação.
- **AC:** "Por que meu FPS caiu?" responde com correlação real e link à evidência; custo operacional ~0.
- **Dependências:** TK-S1712

#### TK-S1812 · L2 — SLM local (Q&A natural offline)
- **Prioridade:** P2 · **Esforço:** XL (13) · **Risco:** 🔴
- **Tasks:** runtime de modelo pequeno (GGUF/ONNX); download/gestão do modelo; RAG sobre Twin + base de troubleshooting; indicação local vs cloud.
- **AC:** responde perguntas comuns offline; footprint do modelo < 2 GB; sem envio de dados.
- **Dependências:** TK-S1811

### TK-F182 · Camada cloud opt-in (L3) + suporte automatizado

#### TK-S1821 · L3 — LLM cloud opt-in com cota por plano
- **Prioridade:** P2 · **Esforço:** L (8) · **Risco:** 🔴
- **Tasks:** integração API LLM; consentimento por sessão; minimização/anonimização de dados; cota por tier; cache.
- **AC:** só envia com opt-in; respeita cota; sem PII; transparente na UI.
- **Dependências:** TK-S1812, TK-S1311

#### TK-S1822 · Suporte técnico automatizado (1º nível) + escalonamento
- **Prioridade:** P3 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** assistente resolve dúvidas comuns; escala a humano com contexto pronto.
- **AC:** reduz tickets de 1º nível; escalonamento leva contexto.
- **Dependências:** TK-S1821

---

## TK-E19 · Cloud opcional (sync)

### TK-F191 · Sincronização opt-in

#### TK-S1911 · Backend de sync stateless + auth
- **Prioridade:** P2 · **Esforço:** XL (13) · **Risco:** 🔴
- **Tasks:** API gateway + auth JWT; sync de perfis/Twin (last-write/CRDT); object storage; tudo opt-in.
- **AC:** com opt-in, perfis/histórico sincronizam entre dispositivos; offline continua 100% funcional.
- **Dependências:** TK-S1312, TK-S1111
- **DoD+:** dados criptografados em trânsito e repouso; opt-out apaga dados na nuvem.

#### TK-S1912 · Multi-dispositivo (limites por plano)
- **Prioridade:** P3 · **Esforço:** M (5) · **Risco:** 🟡
- **Tasks:** vincular N dispositivos por conta; resolver conflito de perfis.
- **AC:** respeita limite por tier; conflitos resolvidos sem perda.
- **Dependências:** TK-S1911

---

## TK-E20 · Enterprise (fleet)

### TK-F201 · Console central & deploy

#### TK-S2011 · Console multi-tenant (frota de PCs)
- **Prioridade:** P2 · **Esforço:** XL (13) · **Risco:** 🔴
- **Tasks:** agente leve + console web; inventário/telemetria agregada por org; políticas centrais de otimização; auditoria central.
- **AC:** admin vê frota, aplica política e audita ações; telemetria agregada anônima.
- **Dependências:** TK-S1911, TK-S1211

#### TK-S2012 · Deploy silencioso (GPO/MDM/MSI) + SSO/SAML
- **Prioridade:** P2 · **Esforço:** L (8) · **Risco:** 🟡
- **Tasks:** MSI silencioso; provisioning via GPO/Intune; SSO/SAML; licença por seats.
- **AC:** TI implanta em massa sem interação; login via SSO; seats controlados.
- **Dependências:** TK-S2011, TK-S1412, TK-S1313

#### TK-S2013 · Relatórios agregados / white-label corporativo
- **Prioridade:** P3 · **Esforço:** M (5) · **Risco:** 🟢
- **Tasks:** relatórios de frota; white-label por org.
- **AC:** admin exporta relatório agregado branded.
- **Dependências:** TK-S2011, TK-S1012
