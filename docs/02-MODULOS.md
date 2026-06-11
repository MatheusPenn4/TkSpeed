# 02 · Módulos

Cada módulo é um **bounded context** (crate Rust) com domínio, casos de uso e adapters próprios. Comunicam-se por contratos (`tk-contracts`) e eventos.

## Diagrama de módulos

```
                         ┌─────────────┐
                         │   TkCore    │  bootstrap • DI • lifecycle • permissões
                         └──┬───┬───┬──┘
            ┌───────────────┘   │   └────────────────┐
            ▼                   ▼                    ▼
     ┌────────────┐     ┌──────────────┐     ┌──────────────┐
     │ TkMonitor  │────▶│  TkAnalyzer  │────▶│  TkOptimizer │
     │ (telemetria)│     │ (diagnóstico)│     │ (aplicação)  │
     └─────┬──────┘     └──────┬───────┘     └──────┬───────┘
           │                   │                    │ snapshot
           │                   ▼                    ▼
           │            ┌──────────────┐     ┌──────────────┐
           │            │  TkBenchmark │     │  TkRollback  │
           │            └──────┬───────┘     └──────────────┘
           │                   │
           ▼                   ▼
     ┌────────────┐     ┌──────────────┐     ┌──────────────┐
     │TkGameBoost │     │   TkReport   │     │   TkStorage  │ (SQLite)
     └────────────┘     └──────────────┘     └──────────────┘
                                ▲                    ▲
                                └──── Digital Twin ──┘ (série histórica)
```

---

## 1. TkCore — Núcleo

**Responsabilidade:** inicialização, container de DI, ciclo de vida, gestão de permissões/elevação, event bus interno, carregamento de módulos (e plugins futuros), comunicação interna.

- `AppContext`: container com repositórios, coletores e config.
- `PermissionBroker`: avalia se uma ação requer elevação; orquestra UAC.
- `ModuleRegistry`: registra módulos e (futuro) plugins.
- `EventBus`: `broadcast` Tokio para telemetria e progresso.
- `Lifecycle`: `bootstrap → ready → running → shutdown` (flush de logs/DB).

## 2. TkMonitor — Monitoramento em tempo real

**Coleta:** CPU (uso/clock/por núcleo), GPU (uso/clock/VRAM/temp), RAM (uso/disponível/paginação), SSD/HDD (IOPS, latência, throughput, temp), temperaturas, voltagem, rede (up/down, latência), FPS (via presentation hook quando disponível).

- `Sampler` por fonte com cadência própria.
- `Adapters`: `LhmAdapter` (LibreHardwareMonitor), `WmiAdapter`, `PdhAdapter` (Performance Counters), `EtwFpsAdapter` (FPS via ETW/PresentMon-like).
- Publica `MetricSample` no event bus + persiste *downsampled* no `tk-storage`.

## 3. TkAnalyzer — Motor de análise

**Detecta:** gargalos (CPU/GPU/RAM/Storage/Network/Thermal), drivers antigos, serviços problemáticos, programas pesados, inicialização lenta, conflitos.

- `BottleneckEngine`: regras + janelas estatísticas sobre telemetria.
- `StartupAnalyzer`: itens de boot (Registry Run, Task Scheduler, Startup folder) + tempo estimado.
- `DriverAuditor`: versão/data de drivers críticos (GPU, chipset, rede, storage).
- `ServiceAuditor`: serviços com alto consumo ou desnecessários (com whitelist de segurança).
- Produz `Diagnosis { findings[], severity, score_impact, recommendations[] }`.

## 4. TkOptimizer — Motor de otimização

**Aplica:** limpeza segura (temp, cache, logs antigos — nunca dados de usuário sem confirmação), ajustes do Windows (visuais, telemetria, indexação), otimização de inicialização, planos de energia, otimização de rede (TCP autotuning, DNS), ajustes gamer.

> **Invariante:** nenhuma ação executa sem `snapshot → log → plano de rollback`. Cada otimização implementa o trait `Optimization` com `preview()`, `apply(tx)`, `revert(snapshot)` e `risk_level`.

- `OptimizationCatalog`: tweaks versionados e categorizados.
- `OptimizationSaga`: `snapshot → apply → verify → commit|compensate`.
- `SafetyGuard`: bloqueia combinações perigosas; exige confirmação por `risk_level`.

## 5. TkGameBoost — Modo Gamer

**Ao detectar jogo (foreground full-screen / lista conhecida):** suspende processos configurados, troca plano de energia, prioriza o jogo (afinidade/prioridade), reduz tarefas secundárias. Restaura tudo ao sair.

- `GameDetector`: ETW/foreground + base de assinaturas de jogos.
- `GameProfile`: por jogo (processos a suspender, plano de energia, overlay on/off).
- Estado sempre **reversível** ao encerrar o jogo (hook de saída + watchdog).

## 6. TkBenchmark — Benchmark interno

**Testes:** CPU (single/multi-thread, criptografia, compressão), RAM (bandwidth/latência), SSD (seq/rand IO), SO (boot proxy, latência de chamadas, responsividade).

- `BenchmarkSuite` determinística e versionada (resultados comparáveis só dentro da mesma versão).
- Gera `BenchmarkResult` + alimenta o **TkSpeed Score** e a comparação antes/depois.

## 7. TkRollback — Reversão completa

**Cria:** snapshots (Registry keys afetadas, estado de serviços, plano de energia, arquivos movidos para quarentena), backups, restauração granular ou total.

- `SnapshotStore`: snapshots imutáveis com hash de integridade.
- `Quarantine`: arquivos "deletados" vão para quarentena com TTL antes de remoção real.
- Integra com Restore Point do Windows para ações de maior impacto.

## 8. TkReport — Relatórios

Exporta **PDF** e **HTML** com: resumo executivo, TkSpeed Score, gargalos, otimizações aplicadas, antes/depois, gráficos. Templates premium e *white-label* (Studio tier).

## 9. TkAI (Futuro)

Assistente que responde "Por que meu FPS caiu?", "Por que meu SSD está lento?", "Qual processo está causando gargalo?" — correlacionando o Digital Twin com regras/modelo. Roda **local-first** (modelo pequeno) com opção de backend remoto opt-in.

---

## Dependências de módulo (camadas)

| Módulo | Depende de | Não depende de |
|---|---|---|
| TkCore | todos (composição) | — |
| TkMonitor | tk-storage, tk-contracts | TkOptimizer |
| TkAnalyzer | TkMonitor (dados), tk-storage | TkOptimizer |
| TkOptimizer | TkRollback, tk-storage | TkMonitor |
| TkGameBoost | TkMonitor, TkOptimizer | TkReport |
| TkBenchmark | tk-storage | UI |
| TkRollback | tk-storage | TkOptimizer |
| TkReport | todos (somente leitura) | — |
