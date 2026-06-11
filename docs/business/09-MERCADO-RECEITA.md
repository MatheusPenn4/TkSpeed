# 09 · Estimativa de Mercado & Projeção de Receita

> ⚠️ **Números ilustrativos** baseados em premissas declaradas (ordens de grandeza públicas + benchmarks de SaaS/freemium). Devem ser validados com dados reais de tração. Servem para dimensionar oportunidade e modelar cenários, não como garantia.

## 1. Dimensionamento de mercado (TAM / SAM / SOM)

**Premissas base:** Windows é a esmagadora maioria dos desktops/notebooks do mundo (base ativa na ordem de **~1,4 bilhão** de dispositivos). PC gaming e profissionais de criação somam centenas de milhões.

| Nível | Definição | Estimativa (ordem de grandeza) |
|---|---|---|
| **TAM** | Todos os PCs Windows que poderiam usar otimização/monitoramento | ~1,4 bi dispositivos |
| **SAM** | Usuários propensos a ferramenta premium (gamers + criadores + power users + técnicos + PMEs) | ~150–250 mi usuários |
| **SOM (3 anos)** | Fatia alcançável com nosso GTM (wedge gamer + BR/LatAm + global early adopters) | ~300 mil – 1 mi instalações |

### Segmentos-alvo

| Segmento | Tamanho relativo | Plano principal | Notas |
|---|---|---|---|
| Gamers entusiastas | dezenas de milhões (global) | Pro | Wedge inicial, viral, alta DAP |
| Criadores/profissionais | dezenas de milhões | Pro/Studio | Estabilidade térmica, relatórios |
| Técnicos/assistências | centenas de milhares (BR) | Studio | White-label = gancho B2B |
| PMEs/empresas | milhões de PCs corporativos | Enterprise | Alto ticket, baixo churn |

## 2. Premissas de monetização

| Parâmetro | Conservador | Base | Otimista |
|---|--:|--:|--:|
| Conversão Free→Pago | 2% | 4% | 6% |
| Mix Pro : Studio (dos pagantes) | 90:10 | 88:12 | 85:15 |
| Preço Pro (anual líquido) | R$ 180 | R$ 199 | R$ 220 |
| Preço Studio (anual líquido) | R$ 600 | R$ 699 | R$ 780 |
| Churn anual pago | 35% | 25% | 18% |

> "Usuários" abaixo = **base instalada total** (majoritariamente Free). Receita vem da fração convertida.

## 3. Projeção de receita (ARR)

### 1.000 usuários (base instalada)
| Cenário | Pagantes | Pro | Studio | **ARR** |
|---|--:|--:|--:|--:|
| Conservador | 20 | 18 × R$180 | 2 × R$600 | **~R$ 4,4 mil** |
| Base | 40 | 35 × R$199 | 5 × R$699 | **~R$ 10,5 mil** |
| Otimista | 60 | 51 × R$220 | 9 × R$780 | **~R$ 18,2 mil** |

*Fase de validação — receita simbólica; objetivo é product-fit, não caixa.*

### 10.000 usuários
| Cenário | Pagantes | **ARR** |
|---|--:|--:|
| Conservador | 200 | **~R$ 44 mil** |
| Base | 400 | **~R$ 105 mil** |
| Otimista | 600 | **~R$ 182 mil** |

### 100.000 usuários
| Cenário | Pagantes | Pro | Studio | **ARR** |
|---|--:|--:|--:|--:|
| Conservador | 2.000 | 1.800 × R$180 | 200 × R$600 | **~R$ 444 mil** |
| Base | 4.000 | 3.520 × R$199 | 480 × R$699 | **~R$ 1,04 mi** |
| Otimista | 6.000 | 5.100 × R$220 | 900 × R$780 | **~R$ 1,82 mi** |

### Camada Enterprise (upside separado)
Não modelada acima. Um único contrato de **500 seats** a ~R$ 120/seat/ano = **~R$ 60 mil ARR**. Poucos contratos PME mudam materialmente o quadro (ticket alto, churn baixo). É o motor de crescimento da fase v2.0.

## 4. Economia unitária (alvos)

| Métrica | Alvo |
|---|---|
| ARPU (blended, base total) | R$ 8–18/usuário/ano |
| ARPPU (por pagante) | ~R$ 215/ano |
| CAC (PLG + afiliados) | < R$ 60 |
| LTV (Pro, churn 25%) | ~R$ 199 / 0,25 ≈ **R$ 796** |
| LTV/CAC | **> 3** (meta > 5 no maduro) |
| Margem bruta | > 85% (custo marginal local ~0; cloud só opt-in) |
| Payback CAC | < 6 meses |

## 5. Alavancas de crescimento de receita

1. **Conversão** — calibrar gating e aha-moment (cada +1pp de conversão em 100k base ≈ +R$ 200k ARR).
2. **Expansão** — Studio→Enterprise; multi-device; add-ons (TkAI cloud, packs de plugins).
3. **Retenção** — Digital Twin cria *lock-in* saudável (histórico insubstituível) → reduz churn → aumenta LTV.
4. **Marketplace de plugins (v2.0)** — revenue share, nova linha de receita.
5. **B2B** — assistências (Studio) e PMEs (Enterprise) elevam ARPPU e estabilizam caixa.

## 6. Leitura para investidores

- Margem de software desktop com custo marginal ~zero (IA local) → **escala muito eficiente**.
- Receita recorrente + base freemium grande = **funil previsível** quando o product-fit for provado.
- Upside assimétrico em **Enterprise** e **marketplace** sem inflar custo do core.
- Risco principal não é técnico (arquitetura resolvida) e sim **conversão/posicionamento** — endereçado pelo GTM e pela diferenciação.
