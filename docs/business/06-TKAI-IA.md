# 06 · Estratégia de IA — TkAI

> Objetivo: transformar o Digital Twin em um **assistente de performance** que explica, recomenda e (no futuro) automatiza decisões — sem comprometer privacidade nem inflar custo.

## 1. IA local ou cloud? → **Híbrido, local-first**

| Camada | Onde roda | Função |
|---|---|---|
| **L1 · Motor de regras + heurísticas** | Local (Rust) | Correlações determinísticas sobre o Twin (gargalo, regressão). Já é "inteligência" sem LLM. |
| **L2 · SLM local** | Local (modelo pequeno) | Linguagem natural: explicar findings, responder perguntas comuns offline. |
| **L3 · LLM cloud (opt-in)** | Servidor | Perguntas complexas, raciocínio sobre logs extensos, suporte técnico avançado. |

**Default = L1 + L2 (local, privado, grátis de operar).** L3 é **opt-in** e exclusivo de planos pagos, com consentimento explícito de envio de dados.

## 2. Modelos recomendados

- **L2 (local):** modelo pequeno quantizado (ex.: 1–3B, classe Phi-mini / Llama 3.2 1–3B / Qwen2.5 1.5B) em formato GGUF, rodando via runtime leve (llama.cpp/ONNX Runtime) com aceleração quando disponível. Footprint alvo: < 2 GB em disco, inferência em CPU/GPU integrada.
- **L3 (cloud):** modelo de fronteira via API (classe Claude/GPT) **apenas** para casos avançados, com prompt enxuto e dados minimizados.
- **Embeddings locais** para busca semântica no histórico/base de conhecimento (RAG sobre o Twin + docs de troubleshooting).

## 3. Arquitetura de inferência (RAG sobre o Digital Twin)

```
Pergunta → Recuperação (séries do Twin + eventos + base de regras)
   → L1 tenta responder com regra determinística
   → senão, L2 (local) gera explicação a partir do contexto recuperado
   → se complexo e usuário opt-in: L3 (cloud) com contexto minimizado
   → resposta + ação sugerida (sempre reversível) + "ver evidências"
```

## 4. Casos de uso do TkAI

- **Análise de gargalos em linguagem natural:** "Por que meu FPS caiu?" → correlaciona evento (driver/otimização) + telemetria.
- **Diagnóstico de lentidão:** "Por que meu SSD está lento?" → checa espaço, latência, tipo, processos de IO.
- **Identificação de processo culpado:** "Qual processo está causando gargalo?" → ranqueia por impacto medido.
- **Recomendações inteligentes:** sugere o plano de otimização com maior ganho esperado para *aquela* máquina.
- **Suporte técnico automatizado:** primeiro nível de suporte; resolve dúvidas comuns; escala para humano com contexto pronto (reduz custo de suporte).
- **Detecção proativa:** alerta de regressão/anomalia antes do usuário perceber.

## 5. Privacidade

- L1/L2 **não enviam nada** para fora.
- L3 exige **opt-in explícito por pergunta ou sessão**; dados minimizados e anonimizados; sem PII.
- Transparência: a UI sempre indica se a resposta foi **local** ou **cloud**.
- Conformidade LGPD/GDPR; logs de IA ficam locais.

## 6. Custo estimado (ordem de grandeza)

| Camada | Custo marginal por uso | Observação |
|---|---|---|
| L1 (regras) | ~R$ 0 | roda local, CPU desprezível |
| L2 (SLM local) | ~R$ 0 op. | custo é só o download do modelo; inferência no device do usuário |
| L3 (LLM cloud) | ~R$ 0,02–0,20 por pergunta complexa | depende do modelo/tokens; mitigado por RAG enxuto e cache |

**Estratégia de custo:** maximizar L1/L2 (custo zero operacional), reservar L3 para o que agrega valor pago. Definir **cota mensal de L3 por plano** (ex.: Pro = N perguntas cloud/mês) para previsibilidade. Margem protegida.

## 7. Roadmap do TkAI

1. **Fase 4.0:** L1 (regras/correlação) + explicações via templates. *(quase sem custo, alto valor)*
2. **Fase 4.1:** L2 SLM local para Q&A natural offline.
3. **Fase 4.2:** L3 cloud opt-in para casos avançados + suporte automatizado.
4. **Fase 4.3:** recomendações preditivas e automação supervisionada (sempre com rollback).

## 8. Guardrails

- TkAI **nunca aplica** mudança sozinho sem confirmação (ou, no modo automático, sempre com snapshot+rollback e log).
- Respostas citam **evidências** (gráficos/eventos do Twin), evitando alucinação.
- Fallback gracioso: sem certeza, recomenda cautela e não inventa solução.
