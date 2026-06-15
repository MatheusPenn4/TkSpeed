# TkSpeed — Brand Guide (V3 · Apex)

Identidade oficial do produto. Instrument-grade / Engenharia de Performance.

## Assinatura oficial da marca
> **TkSpeed**
> Engenharia de Performance

Assinatura única do produto (rail, splash, brand guide, landing interna, metadados do app,
sobre, instalação, documentação). **"Performance" é o termo-marca oficial** — não substituir por
"desempenho". Interface 100% em português brasileiro; loanwords técnicos consagrados permanecem
apenas quando são o termo de mercado (ex.: "Benchmark").
Os tokens vivem em `apps/desktop/src/shared/apex/apex.css`; os componentes de
marca em `apps/desktop/src/shared/apex/branding/`.

## 1. Cor

### Primary Signal — Quantum Mint `#58F2D2`
A cor da **ação e do estado do produto**: nav ativo, CTA primário, foco, "medindo agora",
signal-lock, item verificado. Usar com parcimônia — é luz de instrumento, ≤1 foco por tela.

### Secondary Ion — `#39C7FF`
A cor dos **dados**: TkSpeed Score, traços de benchmark, telemetria, gráficos do History,
sparklines. Vive nos números e curvas, não em botões.

### Gradiente de marca
`linear-gradient(135°, #58F2D2 → #39C7FF)` — usado em UM momento por tela (arco do Score,
Signal-Lock ao travar, logo mark). Nunca como fundo de área grande.

### Substrato e tinta
Grafite anodizado: `#08090C → #0D0F14 → #13161D → #1A1E27 → #222732`. Tinta `#F2F5FA / #9BA6B7 / #5A6473`.

### Semânticas (separadas do sinal)
`--ok #46C88A` · `--warn #E8B84B` · `--risk #EF6E6E` · `--info #6E93FF`.
**O mint NÃO é a cor de sucesso** — "verificado/medido" usa mint + ícone; "sucesso" usa `--ok`.

### Proibições
Sem RGB gamer, neon excessivo, azul SaaS genérico ou verde-sucesso tradicional como acento.

## 2. Tipografia

| Fonte | Uso |
|---|---|
| **Geist** | títulos, navegação, branding, headlines, métricas principais |
| **Geist Mono** | benchmarks, FPS, frametimes, clocks, temperaturas, score técnico, logs, diagnósticos — todo dado, `tabular-nums` |

Self-hosted (`@fontsource/geist-sans` / `geist-mono`, pesos 400/500/600). Nunca CDN (app offline).
Tokens: `--ax-font-display`, `--ax-font-ui` (Geist), `--ax-font-data` (Geist Mono).

## 3. Spacing & layout
Base 4px, ritmo 8px. Raios fechados (engenharia): `4 / 6 / 10 / 14` (`--ax-r-xs..lg`), pílula 999.
Rail 248px (colapsa 64). Content-max 1440. Densidade dupla: confortável (cards) e instrument (dados, linhas 36px).
Elevação por luz (hairline + inset highlight), não drop-shadow pesado.

## 4. Logo & símbolo

- **Símbolo** (`branding/BrandSymbol`): a marca isolada. Proporção ~1.58:1 — dimensionar por ALTURA.
- **Logo completo** (`branding/BrandLogo`): símbolo + wordmark.
- **Lockup** (`branding/BrandHeader`): `[símbolo] TkSpeed / Performance Engineering`.
- **Marca-d'água** (`branding/BrandWatermark`): símbolo grande, opacidade ~3.5%, decorativo.

### Clear space
Margem mínima ao redor do logo/símbolo = **altura do símbolo ÷ 2** em todos os lados.
Nunca encostar texto/borda dentro dessa zona.

### Tamanhos mínimos
Símbolo: ≥16px de altura. Logo completo: ≥20px de altura (abaixo disso, usar só o símbolo).

## 5. Ícone do aplicativo (SO)
Gerado de uma fonte **quadrada 1024×1024** (símbolo centralizado, ~90% do canvas) via `tauri icon`,
produzindo `src-tauri/icons/*` (icon.ico multi-res, pngs, icns). Cobre taskbar, executável,
instalador (NSIS/MSI), atalhos e Menu Iniciar.
> Nota: a arte-fonte do símbolo é retangular (256×162); foi padronizada para quadrado com
> margem transparente. Para nitidez máxima nos tamanhos grandes, fornecer um master quadrado ≥1024.

## 6. Icon rules (UI)
Set próprio (`AxIcon`): stroke 1.5px, grid 24, terminais consistentes, geométrico.
Cor herda de `currentColor`. Ativo/medindo = mint; dados = ion; neutro = tinta.
Nunca misturar com famílias de ícones de terceiros.
