# 13 · TkSpeed Score

Sistema de pontuação proprietário **0–1000** que resume a saúde e o desempenho da máquina numa métrica comparável ao longo do tempo (Digital Twin).

## 1. Categorias e pesos

| Categoria | Peso | O que mede |
|---|---:|---|
| CPU | 16% | desempenho (benchmark) + headroom + throttling |
| GPU | 16% | desempenho + VRAM + térmico |
| RAM | 12% | capacidade vs. uso, bandwidth, pressão/paginação |
| Storage | 12% | tipo (NVMe/SATA/HDD), IOPS, latência, espaço livre |
| Windows | 12% | startup, serviços, telemetria/bloat, atualizações |
| Network | 8% | latência, estabilidade, config TCP |
| Temperatura | 12% | margem térmica, throttling, picos |
| Jogos | 6% | estabilidade de FPS (1% low), stutter |
| Estabilidade | 6% | crashes, erros de SO, uptime saudável |

> Pesos somam 100%. São versionados (mudança de pesos = nova versão de score, para manter comparabilidade histórica).

## 2. Cálculo

Cada categoria produz um **subscore normalizado 0–100** a partir de métricas medidas, comparadas a faixas de referência (não a um "PC ideal" abstrato, mas a faixas realistas por classe de hardware).

```
subscore_i = clamp(0,100, Σ ( norm(metric) * w_metric ))
score_total = round( Σ ( subscore_i/100 * peso_i ) * 1000 )
```

### Exemplo — categoria Temperatura
```
margem_termica = T_throttle - T_media_carga      (°C de folga)
norm_margem    = clamp(0,1, margem_termica / 30)  (30°C de folga = 1.0)
penalidade_throttle = eventos_throttle_por_hora * 0.1
subscore_temp = 100 * max(0, norm_margem - penalidade_throttle)
```

### Exemplo — categoria Storage
```
base = { NVMe:100, SATA_SSD:80, HDD:40 }[tipo]
ajuste_espaco = (espaco_livre% < 10) ? -20 : 0
ajuste_latencia = clamp(-20,0, -(latencia_ms - 0.1)*X)
subscore_storage = clamp(0,100, base + ajuste_espaco + ajuste_latencia)
```

## 3. Classificações

| Faixa | Classificação | Cor |
|---|---|---|
| 0–199 | **Crítico** | `#FF4D4D` |
| 200–449 | **Regular** | `#FFC857` |
| 450–699 | **Bom** | `#00E5FF` |
| 700–899 | **Excelente** | `#00FF88` |
| 900–1000 | **Elite** | gradiente + glow |

## 4. Transparência (anti-caixa-preta)

- A UI mostra o **breakdown por categoria** e o que puxou o score para baixo.
- Cada subscore lista as métricas e como foram normalizadas.
- O score é **explicável**: clicar numa categoria abre os findings que a afetam (liga ao TkAnalyzer).

## 5. Comparabilidade

- Score só é comparado dentro da mesma `score_version` e, idealmente, mesma classe de hardware.
- Antes/depois de otimização usa o mesmo contexto → delta confiável.
- Persistido em `scores.breakdown_json` para o Digital Twin.

## 6. Anti-gaming

- O score não é inflado por "limpar lixo trivial". Limpeza só move storage/Windows marginalmente.
- Ganhos vêm de mudanças que **medem** diferença em benchmark/telemetria, alinhando o número à realidade percebida.
