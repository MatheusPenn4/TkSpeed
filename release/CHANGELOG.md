# Changelog — TkSpeed

## 0.1.0-alpha (2026-06-06) — primeiro Alpha público

Primeira versão distribuível para testes externos.

### Incluído
- **Dashboard** com telemetria real em tempo real (CPU, RAM, Disco) + detecção de hardware.
- **TkSpeed Score** (0–1000) e **análise de gargalos** (CPU/RAM/Storage) com problema → impacto → solução.
- **Performance Lab**: benchmarks CPU/RAM/Storage + hardware ao vivo (GPU via NVML quando disponível) + detector de gargalo.
- **Confidence Engine**: noise floor por máquina, confiança e margem dinâmica — nunca declara ganho/perda no ruído.
- **Captura de FPS** (frame time, 1%/0.1% low) via PresentMon quando presente; demo do pipeline incluída.
- **Centro de Otimizações** (loop fechado, baseado em evidência):
  - Plano de Energia: Alto Desempenho (validado por benchmark de CPU).
  - Limpeza de Temporários / Logs / Cache do Windows Update (espaço liberado, reversível via quarentena).
  - Game Mode / Desativar Game DVR (aplicados e reversíveis; comprove com FPS no jogo).
  - Análise de Inicialização (somente leitura).
- **Snapshots + Rollback** auditável e verificado para toda alteração.
- **Logs** em arquivo (`%APPDATA%\TkSpeed\logs`) + housekeeping do banco.

### Princípios
- Toda otimização: medida, comparada e mantida só com evidência — senão revertida.
- Nada destrutivo; tudo reversível e auditado.

### Limitações conhecidas (Alpha)
- Instalador **não assinado** (SmartScreen aparece — ver LEIA-ME).
- GPU detalhada só em placas **NVIDIA** (via NVML); demais aparecem como "indisponível".
- Captura de FPS real exige **PresentMon** + execução como administrador.
- Temperaturas de CPU dependem do que o Windows expõe (frequentemente "indisponível").
- HAGS e otimizações avançadas ainda não incluídas.
- Dois benchmarks de CPU seguidos podem aquecer a máquina e gerar resultado "inconclusivo"
  (comportamento honesto; não inventa ganho).
