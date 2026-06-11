# 11 · Plano de Escalabilidade

Escalabilidade aqui tem três dimensões: **de código** (manter o produto crescível), **de carga local** (lidar com muitos dados/telemetria na máquina) e **de negócio** (de 1 usuário a centenas de milhares + fleets).

## 1. Escalabilidade de código (arquitetura)

- **Bounded contexts** isolados em crates → equipes paralelas sem colisão.
- **Contratos primeiro** (`tk-contracts`) → frontend e backend evoluem com tipagem garantida.
- **Catálogo de tweaks plugável** (1 arquivo = 1 `Optimization`) → adicionar otimização não toca o core.
- **Adapters de coleta plugáveis** → novo sensor = novo adapter.
- **Feature flags** para liberar capacidades por tier/fase sem branches longas.

## 2. Plugins (Fase 5)

- ABI estável via **WASM** (sandbox) para otimizações/coletores de terceiros.
- Manifesto declarativo (permissões, `risk_level`, pontos de extensão).
- Plugins rodam isolados; não acessam o sistema diretamente — pedem ao host por capabilities auditadas.
- Marketplace com assinatura/curadoria de plugins.

## 3. Escalabilidade de dados local

- SQLite + WAL aguenta milhões de linhas de telemetria com a política de **downsampling/rollup** (s1→s10→m1).
- Índices em `(ts, source, metric)`; consultas do Digital Twin operam sobre rollups, não raw.
- Housekeeping noturno + `VACUUM`; tamanho do DB limitado por retenção configurável.
- Caso extremo (estações 24/7): partição lógica por mês via tabela de rollup mensal.

## 4. Escalabilidade de performance da ferramenta

- Coleta assíncrona com cadências independentes e backpressure (broadcast com lag policy).
- Orçamento rígido: **< 1% CPU idle, < 150 MB RAM, < 1% bateria/h** em monitoramento leve.
- UI renderiza por evento (sem polling); gráficos com virtualização e throttle.

## 5. Escalabilidade de negócio

- **Distribuição:** site + Microsoft Store + afiliados.
- **Licenciamento offline-first** → não exige infra pesada; servidor de ativação é stateless e cacheável (escala horizontal trivial atrás de CDN).
- **Cloud opcional (opt-in, Fase 5):** sync de perfis/Digital Twin entre máquinas; backend stateless + object storage + fila para relatórios pesados.
- **Fleet/B2B (Studio):** agente leve + console central para muitas máquinas (multi-tenant), telemetria agregada.

## 6. Infra de cloud (apenas para features opt-in)

```
[App Desktop] ──(opt-in)──▶ API Gateway ──▶ Auth (stateless JWT)
                                   │
                       ┌───────────┼─────────────┐
                       ▼           ▼             ▼
                 Sync Service  Report Worker   Telemetry Aggr.
                  (CRDT/last-write)  (fila)      (anônima, opt-in)
                       │           │             │
                       ▼           ▼             ▼
                   Postgres    Object Store   ClickHouse/TS DB
```

- Tudo **opt-in**; o produto é 100% funcional offline.
- Stateless onde possível → escala horizontal via autoscaling + CDN.

## 7. Observabilidade (do produto e do negócio)

- Logs locais estruturados; crash reporting opt-in.
- Métricas de adoção/conversão anônimas (opt-in) para guiar roadmap.
- Health budget monitorado no CI (regressão de footprint quebra o build).
