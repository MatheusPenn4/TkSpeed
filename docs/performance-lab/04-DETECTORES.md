# 04 · Detectores de Gargalo (sobre a sessão de benchmark)

Classificam o que **limita** o desempenho durante uma sessão capturada. Operam sobre os
`MetricFrame` agregados + correlação com frametime. Saída: tipo de gargalo + severidade +
evidência (para o usuário entender, no espírito problema→impacto→solução).

> Importante: estes detectores **diagnosticam**, não otimizam. Eles tornam a otimização
> futura *direcionada* (ataca o gargalo certo) e *verificável* (o bound deve mudar após a melhoria).

## Heurísticas (janelas estatísticas da sessão)

| Bound | Condição (sustentada na janela) | Evidência registrada |
|---|---|---|
| **GPU Bound** | GPU usage ≥ ~97% **e** CPU abaixo do teto | %GPU alto + %CPU folgado → "GPU é o limite" |
| **CPU Bound** | CPU (ou 1 núcleo/main thread) saturado **e** GPU < ~90% | %CPU alto + %GPU ocioso → FPS preso pela CPU |
| **RAM Bound** | RAM ≥ ~90% **e** paging (`Pages/sec`) elevado, correlacionado a spikes de frametime | uso RAM + hard faults + stutters |
| **Storage Bound** | latência/queue de disco alta correlacionada a spikes de frametime (hitching de streaming) | latência IO + travadas no carregamento |
| **Thermal Bound** | clock cai enquanto carga alta **e** temperatura perto do limite (throttling) | queda de clock + temp alta = throttle |

### Notas de método
- **Correlação com frametime**: um bound só é afirmado se os spikes de frametime coincidem temporalmente com a condição (ex.: stutter no mesmo instante do hard fault) — evita falso positivo.
- **CPU bound por thread**: jogos costumam ser limitados por 1 thread (main/render); olhar o **núcleo mais quente de uso**, não só a média, reduz falso negativo.
- **Thermal**: precisa de clock + temp (LHM/PDH). Sem esses sensores, o detector térmico fica `indisponível` (honesto), não chuta.
- **Severidade**: proporcional a quão sustentada e quão impactante no 1% low.

## Uso pelo ciclo de prova
1. Detector aponta o bound **antes**.
2. (Futuro) otimização ataca aquele bound.
3. Benchmark **depois** + comparação: o ganho deve aparecer **e** o bound deve aliviar.
   Se o bound não mudou, o "ganho" é suspeito → o Lab questiona, não celebra.
