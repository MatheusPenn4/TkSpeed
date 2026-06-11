# 14 · Digital Twin

O **Digital Twin** é o perfil vivo e histórico da máquina: um modelo persistente que aprende o comportamento normal do PC e permite comparações ao longo do tempo.

## 1. O que registra

- **Identidade & inventário:** hardware detectado, mudanças de componentes/drivers ao longo do tempo.
- **Comportamento:** uso médio de CPU/GPU/RAM por período (idle, carga, jogos).
- **Temperaturas:** médias, picos, eventos de throttling.
- **Desempenho:** scores e benchmarks históricos.
- **FPS:** por jogo, médias e 1% low ao longo do tempo.
- **Eventos:** otimizações aplicadas/revertidas, updates de driver, mudanças de SO.

## 2. Baseline & "normalidade"

O Twin calcula uma **baseline móvel** (médias/desvios por janela) que define o que é normal para *aquela* máquina. Desvios significativos viram **insights**.

```
baseline(metric) = média móvel + desvio (janela 7/30 dias, por contexto: idle|load|game)
anomalia = |valor_atual - baseline| > k * desvio
```

## 3. Detecção de regressão (exemplo central)

```
evento: "driver NVIDIA 552 → 555 em 12/05"
correlação: FPS médio (mesmo jogo, mesma resolução) caiu de 142 → 131 (−8%)
insight: "Após atualização do driver NVIDIA houve queda de 8% no FPS médio.
          Deseja ver detalhes ou reverter o driver?"
```

A correlação é feita ligando a `audit_log`/eventos de inventário com a série temporal de métricas, comparando janelas pré e pós-evento no **mesmo contexto** (jogo/resolução/carga).

## 4. Comparações suportadas

- **Antes/depois** de uma otimização específica.
- **Período × período** (esta semana vs. semana passada).
- **Evento como marco** (antes/depois de um update).
- **Tendência** (o PC está esquentando mais ao longo dos meses? SSD degradando?).

## 5. Armazenamento

- Série temporal em `metric_samples` (rollups m1 retidos 1 ano).
- Eventos em `audit_log` + `hardware_inventory` (first/last seen).
- Scores e benchmarks históricos.
- Tudo **local** (privacidade); sync entre máquinas é opt-in (Fase 5).

## 6. Visualização (tela Histórico)

- Linha do tempo com marcos (updates, otimizações) sobrepostos aos gráficos.
- Sparklines de tendência por categoria do score.
- Cards de insight ("regressão detectada", "melhoria sustentada").

## 7. Base para o TkAI (Fase 4)

O Digital Twin é o **substrato de dados** do assistente: "Por que meu FPS caiu?" é respondido correlacionando eventos e séries do Twin — não com adivinhação, mas com o histórico real da máquina.
