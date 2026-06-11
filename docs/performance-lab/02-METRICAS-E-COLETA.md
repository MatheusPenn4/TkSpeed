# 02 · Catálogo de Métricas & Estratégia de Coleta

## 1. Catálogo de métricas

| Métrica | Unidade | Fonte de coleta | Elevação? | Custo CPU | Observer effect |
|---|---|---|:--:|---|---|
| **FPS médio** | fps | ETW present events (PresentMon-style) | **Sim** (sessão ETW) | baixo | desprezível |
| **1% low** | fps | derivado do frametime (P99) | — | nulo (cálculo) | nenhum |
| **0.1% low** | fps | derivado do frametime (P99.9) | — | nulo | nenhum |
| **Frame time** | ms | ETW present events | Sim | baixo | desprezível |
| **CPU usage** (total/por núcleo) | % | PDH `Processor Information` | Não | muito baixo | desprezível |
| **GPU usage** | % | PDH `GPU Engine` | Não | baixo | desprezível |
| **RAM usage** | % / GB | sysinfo / PDH `Memory` | Não | muito baixo | nenhum |
| **VRAM usage** | MB | PDH `GPU Process Memory` / `GPU Adapter Memory` | Não | baixo | desprezível |
| **Temperaturas** (CPU/GPU) | °C | LibreHardwareMonitor (LHM) | Não* | médio (polling) | baixo |
| **Clock** (CPU/GPU) | MHz | PDH `% Processor Performance` × base / LHM | Não* | baixo-médio | baixo |
| **Paging / hard faults** | /s | PDH `Memory\Pages/sec` | Não | muito baixo | nenhum |
| **Disk latency / queue** | ms / len | PDH `PhysicalDisk` | Não | baixo | desprezível |

\* LHM pode exigir driver de kernel para alguns sensores; rodar sem ele degrada graciosamente (métrica fica indisponível, não quebra).

### Definições rigorosas (lows / frametime)
- Coletamos o **frametime** de cada frame apresentado (present-to-present, em ms).
- **FPS médio** = `1000 / média(frametime)` na janela (após warmup).
- **1% low** = `1000 / P99(frametime)` — i.e., o FPS no percentil 99 de pior frametime.
- **0.1% low** = `1000 / P99.9(frametime)`.
- (Documentamos a convenção escolhida — *percentil de frametime* — para comparabilidade. Resultados só são comparáveis dentro da mesma `suite_version` e mesmas condições.)

## 2. Estratégia de coleta

### Princípios
1. **Não distorcer o que se mede** (observer effect): preferir fontes de baixo overhead (ETW/PDH) e evitar polling agressivo durante captura de jogo.
2. **Cadência por fonte**: frametime é por-evento (ETW); contadores PDH a 250–500 ms; sensores LHM a 1 s (mais caros).
3. **Degradação graciosa**: se uma fonte falhar (sem LHM, GPU sem contador PDH), a métrica fica `indisponível` e o resto continua — nunca quebra a sessão.
4. **Sessão explícita**: coleta de alta fidelidade só roda durante `run_benchmark`/captura, com início/fim claros (abre e fecha o trace ETW).

### Pipeline de coleta (sessão)
```
start():  abre trace ETW (present) [se FPS]; abre queries PDH; inicia LHM (se disponível)
loop:     a cada tick → poll() de cada collector → MetricFrame{ ts, frametime?, cpu, gpu, ram, vram, temp, clock }
          frametimes acumulam num ring buffer dedicado (alta resolução)
stop():   fecha trace/queries; entrega buffers ao BenchmarkEngine
```

### Elevação (ETW para FPS)
Capturar present events via ETW exige uma **sessão de trace em tempo real (admin)**. Por isso:
- FPS/frametime são **opt-in** e só ativam numa **sessão de benchmark de jogo**, com pedido de elevação **explícito e justificado** ("para medir FPS preciso iniciar um trace do Windows").
- Sem elevação: o Lab ainda mede CPU/GPU/RAM/VRAM/temp/clock (sem FPS) — útil para benchmarks sintéticos e diagnóstico.
- A sessão ETW é **somente leitura** (não altera nada no SO) e é encerrada ao fim da captura.

### Custo & orçamento
- PDH + sysinfo: < 0,5% CPU a 500 ms.
- ETW present trace: overhead reportado da técnica PresentMon é desprezível (< 1%).
- LHM: o mais caro; manter a 1 s e desligável.
- **Regra:** o overhead total da coleta durante captura deve ficar **< 2% CPU** para não contaminar a medição (validado em [07](07-VALIDACAO.md)).
