# 06 · Design System — "TkSpeed Aurora"

Estética: **futurista, premium, tecnológica, profissional.** Referências: Tesla UI, Alienware Command Center, MSI Center, Apple, Windows 11 Fluent, dashboards premium do Figma Community. Nada de aparência de software barato.

## 1. Princípios visuais

1. **Dark-first** com profundidade (camadas de vidro sobre fundo quase-preto).
2. **Glassmorphism** comedido + **neon** acentuado (não saturado em excesso).
3. **Dados como herói** — gráficos e gauges são o centro, cromo da UI é discreto.
4. **Motion com propósito** — microinterações que comunicam estado, nunca decorativas demais.
5. **Densidade profissional** — informação rica sem poluição (ritmo de espaçamento consistente).

## 2. Tokens de cor

```
--bg-base:        #080B12   /* fundo app */
--bg-elevated:    #0E131D   /* cards base */
--bg-glass:       rgba(255,255,255,0.04) /* painel de vidro */
--stroke:         rgba(255,255,255,0.08)

--primary:        #00E5FF   /* ciano — ação, dados primários */
--secondary:      #7B61FF   /* roxo — destaque secundário */
--success:        #00FF88   /* verde — bom/saudável */
--danger:         #FF4D4D   /* vermelho — crítico */
--warning:        #FFC857   /* âmbar — atenção */

--text-hi:        #EAF2FF   /* texto primário */
--text-mid:       #9AA7BD   /* secundário */
--text-low:       #5B6678   /* terciário/labels */
```

### Gradientes de marca
```
--grad-primary: linear-gradient(135deg, #00E5FF 0%, #7B61FF 100%);
--grad-health:  linear-gradient(135deg, #00FF88 0%, #00E5FF 100%);
--grad-risk:    linear-gradient(135deg, #FFC857 0%, #FF4D4D 100%);
--glow-primary: 0 0 24px rgba(0,229,255,0.35);
```

### Escala semântica de score (gauge)
`0–199 Crítico` (#FF4D4D) · `200–449 Regular` (#FFC857) · `450–699 Bom` (#00E5FF) · `700–899 Excelente` (#00FF88) · `900–1000 Elite` (gradiente health + glow).

## 3. Tipografia

- **Display/Números:** `Space Grotesk` / `Geist` — para scores e métricas grandes (tabular-nums).
- **UI/Texto:** `Inter` — corpo e labels.
- **Mono:** `JetBrains Mono` — valores técnicos, logs.

Escala: `12 / 13 / 14 / 16 / 20 / 28 / 40 / 56` px. Pesos: 400/500/600/700.

## 4. Espaçamento & raio

- Grid base **4px**: `4 8 12 16 24 32 48 64`.
- Raio: `sm 8` · `md 12` · `lg 16` · `xl 24` · `full 999`.
- Sombra de elevação suave + glow para elementos ativos.

## 5. Efeitos

- **Glass panels:** `backdrop-filter: blur(20px)` + borda `--stroke` + leve gradiente interno.
- **Neon edge** em elementos de foco/ativos (`box-shadow: --glow-primary`).
- **Partículas sutis** no fundo do Dashboard (canvas, baixa opacidade, respeitando `prefers-reduced-motion`).
- **Gráficos em tempo real** com easing e área preenchida em gradiente.
- **Microinterações:** hover lift, ripple discreto, número que "conta" até o valor (count-up), gauges animando com spring.
- **Motion tokens:** `fast 120ms` · `base 200ms` · `slow 400ms`, easing `cubic-bezier(0.22, 1, 0.36, 1)`.

## 6. Componentes (packages/ui)

| Componente | Uso |
|---|---|
| `GlassPanel` | container base de vidro |
| `MetricCard` | métrica + sparkline + delta |
| `ScoreGauge` | gauge radial 0–1000 com classificação |
| `RealtimeChart` | série temporal animada (CPU/GPU/temp) |
| `BottleneckBadge` | severidade colorida |
| `RadialMeter` | uso de CPU/GPU/RAM |
| `Sidebar` / `NavRail` | navegação lateral |
| `TitleBar` | barra custom (Tauri, sem chrome nativo) |
| `Button` (primary/ghost/danger) | ações |
| `Toggle`, `Slider`, `Select` | controles |
| `Toast`, `ConfirmDialog` | feedback / confirmação de risco |
| `BeforeAfter` | comparação de benchmark |
| `Timeline` | histórico/Digital Twin |

## 7. Acessibilidade

- Contraste AA mínimo no texto sobre vidro (overlay garante legibilidade).
- `prefers-reduced-motion` desliga partículas e count-ups.
- Navegação por teclado e foco visível (neon edge).
- Cor nunca é o único portador de significado (ícone + label).

## 8. Layout base (AppShell)

```
┌──────────────────────────────────────────────────┐
│  TitleBar (drag region, min/max/close)            │
├──────┬─────────────────────────────────────────────┤
│ Nav  │  Header (título da tela + ações rápidas)    │
│ Rail │ ───────────────────────────────────────────│
│      │  Conteúdo (grid de GlassPanels)             │
│ 72px │                                              │
└──────┴─────────────────────────────────────────────┘
```
