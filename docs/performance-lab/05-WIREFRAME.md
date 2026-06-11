# 05 · Wireframe — Tela "Performance Lab"

Baixa fidelidade. Estética Aurora (glass + gráficos em tempo real). Foco em **evidência**.

## Tela principal

```
┌─ Performance Lab ─────────────────────────────────────────────────────────┐
│  Meça antes. Otimize depois. Prove o ganho.        [Capturar Baseline ▾]   │
├───────────────────────────────────────────────────────────────────────────┤
│  SESSÃO DE BENCHMARK                                                        │
│  Tipo: ( ● Jogo  ○ CPU  ○ RAM  ○ Disco )   Alvo: [ game.exe ]              │
│  Condições: 1920×1080 · Preset Alto · Plano: Alto Desempenho   [Editar]    │
│  Runs: [5]  Warmup: [10s]  Duração: [60s]            [▶ Iniciar Benchmark]  │
│                                                                             │
│  ┌─ Frametime ao vivo (ms) ──────────────────────────────────────────────┐ │
│  │  16.7 ┊      ╱╲      ╱╲                              ← linha 60fps     │ │
│  │       ┊╲╱╲╱╱  ╲╱╲╱╲╱  ╲╱╲╱╲   spikes destacados em vermelho            │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│   FPS méd  1% low  0.1% low   CPU   GPU   RAM   VRAM   Temp   Clock          │
│    142      118      96       54%   97%   61%   7.2GB  71°C   4.6GHz         │
│                                                                             │
│   VEREDITO: ● GPU Bound (97% GPU / CPU folgada)  → otimização deve mirar GPU│
└───────────────────────────────────────────────────────────────────────────┘
```

## Comparação (antes vs depois)

```
┌─ Comparação ──────────────────────────────────────────────────────────────┐
│  Antes: "limpo"  ▸  Depois: "perfil gamer"   ·  5 runs · mesmas condições   │
├───────────────────────────────────────────────────────────────────────────┤
│  Métrica        Antes      Depois     Δ           Veredito                  │
│  FPS médio      138.0      142.4     +3.2% ±1.4%  ▲ Ganho                    │
│  1% low         108.0      118.0     +9.3% ±1.1%  ▲ Ganho (sentido!)        │
│  0.1% low        82.0       96.0    +17.1% ±2.0%  ▲ Ganho                    │
│  Frame time P99  9.3ms      8.5ms    −8.6% ±1.2%  ▲ Ganho                    │
│  Temp máx        73°C       71°C     −2°C  ±1°C    ◦ Sem alteração (ruído)   │
│  ─────────────────────────────────────────────────────────────────────────│
│  ✔ Ganho comprovado no 1% low (+9.3%, ±1.1%, 5 runs, mesmas condições)      │
│                                            [Exportar evidência] [Salvar]    │
└───────────────────────────────────────────────────────────────────────────┘
```

## Histórico / Baselines

```
┌─ Baselines & Sessões ─────────────────────────────────────────────────────┐
│  ◷ 06/06 "limpo de fábrica"   Score 700 · GPU RTX · driver 552             │
│  ◷ 06/06 "perfil gamer"       Score 742 · mesmo HW                         │
│  Linha do tempo de 1% low ▁▂▃▅▆  (alimenta o Digital Twin)                  │
└───────────────────────────────────────────────────────────────────────────┘
```

## Princípios de UX
- Cada número de ganho aparece **com margem de erro e nº de runs** — nunca um "+X%" pelado.
- "Sem alteração" é exibido com honestidade (cinza, "dentro do ruído").
- Botão **"Exportar evidência"** gera um pacote (condições + runs + estatística) — base do futuro relatório.
- Se faltar sensor (sem FPS por falta de elevação, sem temp por falta de LHM), a célula mostra "indisponível", não um palpite.
