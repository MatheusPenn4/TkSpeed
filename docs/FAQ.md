# TkSpeed 0.1.0-rc1 — FAQ Beta

**Versão:** 0.1.0-rc1 · **Público:** Beta Testers

---

## Instalação e início

**O app não abre depois de instalar. O que fazer?**  
Certifique-se de que o Windows está atualizado (Windows 10 22H2 ou superior / Windows 11). O TkSpeed usa WebView2, que é instalado automaticamente no Windows 11. No Windows 10, pode ser necessário instalar o WebView2 Runtime separadamente via Microsoft.

**Preciso desinstalar uma versão anterior antes de instalar a nova?**  
Sim. Use "Adicionar ou remover programas" no Windows para desinstalar a versão anterior antes de instalar o RC1.

**O instalador pede permissão de administrador. Isso é normal?**  
Sim. O TkSpeed precisa de acesso ao registro do Windows e ao sistema de benchmarks para funcionar corretamente. O app em si não precisa rodar como administrador depois de instalado.

---

## Central de Comando

**O que é o TkSpeed Score?**  
É uma pontuação de 0 a 1000 que representa o desempenho geral da sua máquina. É calculada a partir de múltiplos benchmarks (CPU, RAM, Storage) combinados. Uma pontuação mais alta indica um sistema mais responsivo.

**O botão "Analisar" fica desabilitado. Por quê?**  
O diagnóstico completo exige que o app esteja rodando como administrador, ou que o serviço de backend esteja disponível. Tente reiniciar o app. Se o problema persistir, use o botão "Exportar diagnóstico" e nos envie o log.

**O que faz o "Otimizar Agora"?**  
O Auto Pilot analisa sua máquina, seleciona as otimizações marcadas como "Seguro" e as aplica automaticamente — uma a uma, medindo o desempenho antes e depois de cada uma. Otimizações sem ganho comprovado são revertidas automaticamente. Nada é aplicado sem evidência.

**Por que o Consultor Inteligente não mostra recomendações?**  
O Consultor precisa de pelo menos um benchmark concluído para avaliar sua máquina. Acesse o Laboratório de Performance, execute um benchmark (CPU é suficiente) e volte para a Central de Comando.

---

## Laboratório de Performance

**Quanto tempo leva o benchmark?**  
- CPU: ~15–30 segundos  
- RAM: ~20–40 segundos  
- Storage: ~30–60 segundos (depende do tipo — SSD é mais rápido que HDD)  
- Completo: ~2–3 minutos

**O que é "Confiança da medição"?**  
É a porcentagem de certeza estatística dos resultados. Uma confiança acima de 80% indica que os dados são estáveis e representativos. Abaixo disso, o sistema indica variância alta — útil como informação, mas menos preciso para decisões.

**O painel de GPU aparece como "Indisponível". É um bug?**  
Não. Nesta versão, o monitoramento de GPU em tempo real funciona apenas com GPUs NVIDIA via NVML. Suporte para AMD será adicionado em versão futura.

**Como uso a Captura de FPS?**  
A captura real de FPS requer o PresentMon. Para testar sem um jogo aberto, use o botão "Demo sem jogo", que simula o pipeline completo de captura. Para captura real: baixe `PresentMon-x64.exe`, coloque em `%APPDATA%\TkSpeed\tools` e rode o app como administrador.

---

## Central de Otimizações

**O que significa "Seguro", "Moderado" e "Avançado"?**  
- **Seguro:** Mudanças revertíveis com impacto mínimo no sistema. Recomendado para qualquer usuário.  
- **Moderado:** Ajustes que podem afetar o comportamento de alguns aplicativos. Reversível.  
- **Avançado:** Mudanças mais profundas no sistema. Use somente se entender as implicações. Sempre reversível.

**A otimização foi "Revertida". O sistema fez algo errado?**  
Não — é exatamente o comportamento correto. O TkSpeed mediu o desempenho antes e depois, e concluiu que não houve ganho real. Para proteger seu sistema, a mudança foi desfeita automaticamente. Nenhuma ação sua é necessária.

**Posso aplicar a mesma otimização mais de uma vez?**  
Sim. Cada execução é medida e registrada independentemente.

---

## Gerenciador de Inicialização

**O que são apps de "Alto impacto"?**  
Apps conhecidos por consumir recursos significativos durante o boot (ex.: Discord, Steam, Spotify). Desabilitá-los pode reduzir o tempo de inicialização do Windows em 2–4 segundos por app.

**A estimativa de economia no boot é garantida?**  
Não. É uma estimativa baseada no impacto típico daquele app em máquinas similares. O tempo real pode variar dependendo do seu hardware.

**O botão "Desabilitar" aparece desabilitado (cinza) em alguns apps. Por quê?**  
Apps marcados como "Sistema" foram instalados com privilégios de administrador e só podem ser desabilitados com permissões elevadas. Eles aparecem com o rótulo "Requer admin".

**Como posso reverter um app de inicialização desabilitado?**  
Acesse a Central de Restauração. Cada desabilitação cria automaticamente um snapshot reversível. Clique em "Reverter" para restaurar o app ao estado original.

---

## Central de Restauração e Segurança

**O que é um "Snapshot"?**  
É uma fotografia do estado do sistema no momento em que uma mudança foi aplicada. O TkSpeed cria snapshots automaticamente antes de qualquer otimização ou desabilitação de app de inicialização. Você pode reverter para qualquer snapshot a qualquer momento.

**O que acontece se eu reverter uma otimização?**  
O sistema desfaz exatamente as mudanças que foram aplicadas e restaura os valores anteriores. O processo é verificado por integridade e é seguro.

**Minhas configurações são enviadas para algum servidor?**  
Não. Todos os dados (benchmarks, snapshots, otimizações) ficam armazenados localmente em `%APPDATA%\TkSpeed`. Nenhum dado é enviado para servidores externos.

---

## Problemas e suporte

**Como exporto um log de diagnóstico?**  
Na Central de Comando, há uma opção "Exportar diagnóstico" no menu do app. O arquivo gerado contém informações de hardware e logs de operação — úteis para reportar bugs.

**O app travou. O que fazer?**  
Feche pelo Gerenciador de Tarefas (Ctrl+Shift+Esc → procure "TkSpeed") e reabra. Os dados persistem — nada é perdido.

**Onde reporto problemas encontrados no beta?**  
Envie um e-mail para o time com: descrição do problema, tela afetada, screenshot (se possível) e o log de diagnóstico exportado. Consulte o checklist de beta tester para o formato completo.
