# 03 · Catálogo Inicial de Otimizações

Classificação: **SAFE** (sem risco) · **MODERATE** (snapshot obrigatório) ·
**ADVANCED** (desabilitado por padrão) · **EXPERIMENTAL** (oculto, usuários avançados).
Nenhuma alteração destrutiva. Coluna **Validação** = como o ganho/efeito é provado (honesto).

## SAFE (aplicam direto, sempre com snapshot/quarentena)

| id | nome | reversibilidade | impacto honesto | validação | sucesso |
|---|---|---|---|---|---|
| `cleanup.temp_files` | Limpeza de temporários | quarentena (TTL) | libera espaço; **não** aumenta FPS | SpaceFreed | bytes liberados > 0; nada do usuário tocado |
| `cleanup.wu_cache` | Cache do Windows Update | quarentena | libera espaço | SpaceFreed | bytes liberados > 0 |
| `cleanup.old_logs` | Logs antigos | quarentena | libera espaço | SpaceFreed | bytes liberados > 0 |
| `startup.analyze` | Análise de apps de inicialização | n/a (só leitura) | menor tempo de boot **potencial** | None (relatório) | lista itens + impacto estimado |
| `storage.recommend` | Recomendações de armazenamento | n/a | orienta (SSD, espaço livre) | None | recomendações geradas |

> SAFE de limpeza usa **quarentena** (nunca exclusão imediata) — recuperável por TTL.
> `startup.analyze` e `storage.recommend` **não alteram nada**: apenas recomendam (entram no Advisor).

## MODERATE (snapshot obrigatório + confirmação)

| id | nome | reversibilidade | impacto honesto | validação | rollback |
|---|---|---|---|---|---|
| `energy.power_plan_high` | Plano de energia Alto Desempenho | full (GUID anterior) | mais responsividade; possível +CPU sustentado | Benchmark `cpu-1.0.0` | se ganho ≤ margem → reverte |
| `energy.game_mode` | Game Mode do Windows (HKCU) | full (registry) | foco de recursos em jogo | Benchmark `fps-1.0.0`* | se não-significativo → reverte |
| `services.optional_off` | Desativar serviços **opcionais** (whitelist) | full (StartType anterior) | menos uso de fundo | Benchmark `cpu-1.0.0` + estabilidade | reverte se instabilidade |
| `energy.usb_selective_suspend_off` | Suspensão seletiva USB off | full | menos micro-stutter de periférico | Benchmark `fps-1.0.0`* | reverte se não-significativo |

\* quando há captura de FPS disponível (PresentMon); senão usa Score/estabilidade e marca evidência parcial.
> `services.optional_off` opera **somente** sobre uma **whitelist curada** de serviços
> reconhecidamente opcionais; **blacklist** protege tudo crítico do SO.

## ADVANCED (desabilitado por padrão; exige opt-in + confirmação de risco)

| id | nome | reversibilidade | impacto honesto | validação |
|---|---|---|---|---|
| `registry.documented_tweaks` | Tweaks de registro **documentados** (HKCU) | full (snapshot) | varia por tweak (cada um documentado) | Benchmark associado |
| `process.affinity` | Afinidade de processos (por sessão) | full (não persistente) | pode ajudar CPU-bound específico | Benchmark `cpu`/`fps` |
| `scheduler.tweaks` | Ajustes de scheduler/timer | full (snapshot) | latência/responsividade | Benchmark `fps`/latência |
| `memory.advanced` | Ajustes avançados de memória | RestorePoint recomendado | varia; risco maior | Benchmark + estabilidade |

> ADVANCED: `enabled_by_default=false`. Aplica só com opt-in explícito, snapshot e, quando
> o risco pedir, **ponto de restauração do Windows** como rede extra.

## EXPERIMENTAL (oculto)
`hidden=true`, fora da UI padrão; acessível só por flag de usuário avançado. Mesmo pipeline,
mesma exigência de evidência. (Nenhum item publicado no primeiro lote.)

## Regra de ouro do catálogo
Cada otimização declara **impacto honesto** + **validação real**. O Engine **só exibe "ganho"**
se o benchmark associado provar (Confidence Engine). Limpezas mostram **espaço liberado**, não FPS.
Itens sem validação de performance entram como **recomendação/relatório**, nunca como "ganho medido".
