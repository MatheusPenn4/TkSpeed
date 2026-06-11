# 05 · Arquitetura de Plugins

> Projetada desde o início (mesmo que entregue só na Fase 5) para que o core já nasça com as costuras de extensibilidade — evitando dívida técnica.

## 1. Objetivos

- **Módulos opcionais** — otimizações, coletores e relatórios entregues como plugins.
- **Marketplace futuro** — distribuição e (possível) monetização de plugins.
- **Plugins assinados** digitalmente — confiança e proveniência.
- **Sandbox** — plugin não acessa o SO diretamente; pede capabilities ao host.
- **WASM** — ABI estável, portável, isolado.
- **SDK para terceiros** — DX de primeira para a comunidade.

## 2. Tipos de extensão (pontos de plugin)

| Tipo | Estende | Exemplo |
|---|---|---|
| `Optimization` | catálogo do TkOptimizer | tweak novo de rede |
| `MetricCollector` | TkMonitor | sensor específico de placa |
| `Analyzer Rule` | TkAnalyzer | regra de gargalo custom |
| `Benchmark Suite` | TkBenchmark | teste especializado |
| `Report Template` | TkReport | layout white-label |
| `Game Profile Pack` | TkGameBoost | perfis curados |

## 3. Por que WASM

- **Isolamento**: roda em sandbox, sem acesso direto a memória/SO do host.
- **Portável e estável**: ABI bem definida; não quebra a cada update do core.
- **Multilinguagem**: terceiros escrevem em Rust, AssemblyScript, Go (TinyGo), etc.
- **Determinístico**: bom para testes e segurança.

> Otimizações que exigem privilégio **não executam código nativo arbitrário**: o plugin declara *intenções* (ex.: "definir chave X = Y") que o host valida, captura snapshot e aplica via `tk-platform-win`. O plugin nunca toca o Registry diretamente.

## 4. Modelo de capabilities

```
Plugin (WASM, sandbox)
   │  só pode chamar funções do Host API expostas
   ▼
Host API (capability-gated)
   ├─ read_metrics(scope)         [cap: metrics.read]
   ├─ propose_registry_change()   [cap: optimize.registry]  → host valida + snapshot
   ├─ propose_service_change()    [cap: optimize.services]
   ├─ emit_finding()              [cap: analyze.report]
   └─ render_template(data)       [cap: report.render]
```

- Cada plugin traz um **manifesto** declarando capabilities, `risk_level`, pontos de extensão e metadados.
- O usuário **aprova as capabilities** na instalação (como permissões de app mobile).
- Capabilities perigosas (`optimize.*`) sempre passam pela saga com snapshot — **a regra de rollback obrigatório vale também para plugins**.

## 5. Manifesto (conceito)

```
plugin.toml
  id            = "com.acme.net-tuner"
  name          = "ACME Network Tuner"
  version       = "1.2.0"
  api_version   = "1"               # ABI do host
  entry         = "plugin.wasm"
  capabilities  = ["metrics.read", "optimize.registry"]
  risk_level    = "moderate"
  signature     = "<assinatura Ed25519 do publisher>"
```

## 6. Assinatura & confiança

- Todo plugin é **assinado** pelo publisher; o host valida a assinatura.
- Níveis de confiança: **Oficial** (TkSpeed) · **Verificado** (publisher revisado) · **Comunidade** (aviso explícito).
- Marketplace faz **curadoria/revisão** automática + manual de plugins `moderate/advanced`.
- Revogação: plugin comprometido entra em lista de revogação (bloqueado no próximo check).

## 7. Ciclo de vida

```
descoberta (marketplace) → instalação (aprovar capabilities) →
carregamento (sandbox WASM) → execução (via Host API) →
atualização (assinada) → revogação/remoção (limpa estado)
```

## 8. SDK para terceiros

- **Crate/lib `tkspeed-plugin-sdk`** com tipos do Host API, macros de manifesto e helpers de teste.
- **CLI**: `tkspeed-plugin new | build | test | sign | publish`.
- **Simulador local**: roda o plugin contra telemetria gravada, sem mexer no SO real.
- **Docs + exemplos**: template "hello optimization" e "custom collector".

## 9. Impacto no core hoje (para não gerar dívida)

Mesmo sem marketplace agora, o core já adota:
- `Optimization`/`MetricCollector` como **traits** (plugins implementam as mesmas).
- `ModuleRegistry` em `tk-core` para registro dinâmico.
- `tk-contracts` versionado = base da `api_version`.
- Saga com snapshot aplicável a qualquer origem de mutação.

Assim, transformar "tweak interno" em "plugin externo" é, no futuro, principalmente empacotar — não reescrever.
