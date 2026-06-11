# TkSpeed Alpha — Matriz de Testes

Objetivo: cobrir combinações reais de hardware/SO. Cada testador preenche **uma linha**
da matriz (a config dele) + roda o **roteiro funcional** abaixo.

## Matriz de cobertura (marque o que for testado)
| Dimensão | Variações a cobrir no Alpha |
|---|---|
| SO | ☐ Windows 10 (22H2) · ☐ Windows 11 |
| CPU | ☐ Intel · ☐ AMD |
| GPU | ☐ NVIDIA RTX · ☐ NVIDIA GTX · ☐ AMD RX · ☐ iGPU (Intel/AMD) |
| Disco do sistema | ☐ SSD NVMe · ☐ SSD SATA · ☐ HDD |
| Forma | ☐ Notebook · ☐ Desktop |

> Meta mínima: pelo menos 1 ✔ em cada linha (idealmente Intel+NVIDIA notebook, AMD+RX desktop,
> e um HDD/Win10 antigo para o pior caso).

## Roteiro funcional (rodar em CADA máquina)
| # | Passo | Esperado | OK? |
|---|---|---|---|
| 1 | Instalar pelo `-setup.exe` (passar pelo SmartScreen) | Instala, cria atalho, abre | ☐ |
| 2 | Abrir o app | Dashboard carrega < 2s; hardware correto no cabeçalho | ☐ |
| 3 | Telemetria ao vivo | CPU/RAM/Disco atualizam ~1s; valores coerentes c/ Gerenciador de Tarefas | ☐ |
| 4 | "Analisar Agora" | Score + gargalos aparecem | ☐ |
| 5 | Performance Lab → CPU | Benchmark roda; mostra confiança | ☐ |
| 6 | Performance Lab → RAM e Storage | Rodam e mostram métricas | ☐ |
| 7 | Performance Lab → Hardware ao vivo | GPU (NVIDIA) com uso/VRAM/temp; ou "indisponível" honesto | ☐ |
| 8 | Otimização: Plano de Alto Desempenho | Bench antes/depois + decisão (mantido/revertido) | ☐ |
| 9 | Otimização: Limpeza de Temporários | "Liberados X MB"; **Reverter** restaura arquivos | ☐ |
| 10 | Otimização: Game Mode | Aplica + "comprove com FPS"; **Reverter** restaura registro | ☐ |
| 11 | Proteção: Autoteste | Os 4 passos PASSAM | ☐ |
| 12 | Controles de janela | Minimizar/Maximizar/Fechar funcionam | ☐ |
| 13 | Fechar e reabrir | Histórico/sessões persistem | ☐ |
| 14 | Exportar diagnóstico | `Export-TkSpeedDiagnostic.ps1` gera o zip | ☐ |
| 15 | Desinstalar | Some dos Apps; sem erro | ☐ |

## Casos de atenção (anotar comportamento)
- **Sem NVIDIA**: GPU deve dizer "indisponível" (não travar).
- **HDD**: Storage bench muito mais lento — normal; detector deve indicar HDD.
- **Notebook**: Plano de Alto Desempenho pode aquecer e dar "inconclusivo" no 2º bench — é honesto.
- **WU cache cleanup sem admin**: libera pouco/zero (arquivos bloqueados) — não deve travar.
- **PresentMon ausente**: "Capturar FPS" deve dar erro claro, não crash.

Resultado de cada máquina → enviar `TkSpeed-Diagnostic.zip` + `FEEDBACK.md`.
