# AGENTS.md - PrysmView

## Regras de codigo e seguranca

### Escopo de alteracao
- O agente so pode alterar arquivos listados em `arquivos_permitidos` da tarefa em `docs/plan/tasks.yaml`.
- Qualquer alteracao fora dessa lista eh bloqueada pelo script de orquestracao.
- Nenhuma tarefa começa com dependencia aberta.

### Tamanho das tarefas
- No maximo 5 arquivos por tarefa.
- Cerca de 300 linhas de diff por tarefa.
- Acima disso, o script recusa e pede para dividir.

### Plano antes do codigo
- O agente descreve em ate 10 linhas o que vai fazer, arquivos que vai alterar e riscos.
- Nao escreva codigo ate o plano ser aprovado pelo orquestrador.

### Tentativas
- Maximo de 3 ciclos com erro por tarefa.
- Apos 3 tentativas falhas, a tarefa fica "bloqueada" e vai para o humano.

### Aprovacao humana obrigatoria
- Merge na main.
- Mudanca em protocolo, criptografia ou seguranca.
- Mudanca em ADR.
- Dependencia nova.
- Qualquer coisa tocando seguranca.

### Sem auto-aprovacao
- O agente que escreveu o codigo nunca eh o que revisa.
- O revisor de seguranca checa o diff contra o threat model.

### Linguagens suportadas
- Rust: cargo fmt --check, cargo clippy -- -D warnings, cargo test, cargo audit
- Go: gofmt -l ., go vet ./..., govulncheck ./..., gosec ./...

# ESPECIFICAÇÃO COMPLETA: PRYSMVIEW

Você é um engenheiro sênior trabalhando no PrysmView. Este documento é a fonte de verdade do projeto. Se uma tarefa conflitar com ele, pare e pergunte. Não improvise arquitetura, protocolo, dependências ou comportamento de segurança.

## 0. Missão

PrysmView é um software de acesso remoto desktop para PDVs (caixas de loja) rodando Windows 10 e Windows 11. Um operador (técnico ou central de suporte) visualiza e controla o PDV de forma remota. O software deve ser:
- rápido: latência percebida baixa, fluidez de vídeo aceitável em rede de loja;
- leve: uso baixo de CPU, memória e disco no PDV, que costuma ter hardware modesto;
- seguro: criptografia fim a fim, consentimento visível no PDV, auditoria de sessões;
- de custo zero: nenhuma ferramenta, biblioteca, serviço, domínio ou certificado pago.

Interface visual inspirada no TeamViewer, em estilo limpo e funcional.

## 1. Atores

- Operador: pessoa que inicia a sessão a partir de outro computador (viewer).
- Usuário da loja: pessoa que está no PDV. Deve sempre ver quando há acesso e pode recusá-lo ou encerrá-lo.
- Administrador: responsável por cadastrar PDVs, operadores e revogar acessos (via API e banco).

## 2. Fluxo de uma sessão (ordem exata)

1. O host (agente no PDV) inicia como serviço Windows e gera, na primeira execução, um par de chaves Ed25519. A chave pública é a identidade do PDV. A privada é protegida por DPAPI (escopo de máquina).
2. O host se registra na sinalização com: chave pública, nome da loja, número do PDV, versão do agente e status.
3. O operador faz login na API de sinalização (token opaco de vida curta).
4. O operador escolhe um PDV. A API verifica se esse operador tem permissão de acesso a esse PDV.
5. A API informa ao host, por canal de sinalização, que há uma solicitação de sessão, com id do operador e horário.
6. Se o operador não está na lista de confiança do PDV, o host mostra uma janela de consentimento no PDV com nome do operador, motivo informado e botões Aceitar e Recusar. Sem resposta em 60 segundos, a sessão é recusada.
7. Ao aceitar, o host gera um código de sessão de 8 dígitos, com validade de 120 segundos e no máximo 3 tentativas, e o exibe na própria janela do PDV. O operador digita esse código no viewer. O código é verificado por HMAC no host, com comparação em tempo constante.
8. O viewer conecta-se ao host por iroh (QUIC com TLS 1.3), usando a chave pública do PDV como endereço.
9. Dentro da conexão, o primeiro stream é de controle. Nele ocorrem: handshake de versão, validação do código de sessão e definição de permissões (vídeo, entrada, arquivos, clipboard).
10. Os streams de vídeo, entrada, arquivos e chat são abertos conforme as permissões.
11. Durante a sessão, a janela de indicação no PDV fica visível e há um botão de encerramento imediato.
12. Ao encerrar (por qualquer lado, por queda ou por tempo máximo de 4 horas), o host solta todas as teclas e modificadores, descarta chaves de sessão e grava o registro de auditoria.

## 3. Escopo

Incluído na versão 1:
- visualização da tela de um monitor por vez, com seleção de monitor;
- controle de mouse e teclado;
- transferência de arquivos com verificação de hash SHA-256;
- área de transferência compartilhada (texto, com limite de 1 MB), desativável;
- chat de texto simples;
- reinício do PDV com reconexão automática;
- Ctrl+Alt+Del a partir do viewer;
- atualização automática assinada;
- auditoria de sessões.

Fora de escopo na versão 1 (não implementar, nem parcialmente):
- gravação de sessão em vídeo;
- acesso sem consentimento no PDV, em nenhuma hipótese;
- acesso a PDVs por navegador;
- áudio;
- múltiplas sessões simultâneas no mesmo PDV;
- gerenciamento de múltiplos monitores ao mesmo tempo (só um por vez);
- suporte a macOS e Linux no host.

## 4. Premissas de confiança (threat model resumido)

- O PDV é um ativo valioso e pode estar em rede hostil (Wi-Fi compartilhado, rede da loja mal configurada).
- A rede entre operador e PDV é considerada hostil: qualquer pacote pode ser interceptado, alterado ou reenviado.
- O servidor de sinalização pode ser comprometido. Por isso ele nunca vê conteúdo de sessão (tela, teclas, arquivos), apenas metadados de conexão.
- O relay vê apenas tráfego criptografado, nunca o conteúdo.
- Um operador legítimo pode ter credencial vazada. Por isso há código de sessão, consentimento local e auditoria.
- O usuário local da loja pode tentar recusar ou encerrar acessos. O sistema deve respeitar essa decisão imediatamente.
- Um atacante com acesso físico ao PDV está fora de escopo.

## 5. Arquitetura de componentes

Processos:
- prysmview-svc: serviço Windows (LocalSystem). Faz captura, injeção de entrada, rede e criptografia. É o único processo com privilégios elevados.
- prysmview-tray: aplicativo da bandeja, rodando na sessão do usuário logado. Mostra janela de consentimento, indicação de sessão e botão de encerrar. Não tem privilégios elevados e se comunica com o serviço por named pipe local com ACL restrita ao usuário logado e a SYSTEM.
- prysmview-viewer: aplicativo do operador (eframe/egui).
- prysmview-relay: binário iroh-relay rodando na VM Oracle, via systemd.
- prysmview-api: API de sinalização em Go, na VM Oracle, via systemd.

Comunicação:
- host <-> viewer: iroh/QUIC, TLS 1.3, chave pública como endereço.
- host <-> sinalização: HTTPS (TLS 1.3) com autenticação por assinatura Ed25519 de cada requisição, usando nonce e timestamp para evitar replay.
- serviço <-> tray: named pipe local, mensagens com o mesmo formato de versão do protocolo.

## 6. Stack tecnológica

Regra geral: fixe versões no Cargo.lock e no go.sum. Toda dependência nova exige entrada em docs/dependencias.md com licença, motivo e alternativa considerada. Licenças aceitas: MIT, Apache-2.0, BSD-2/3, ISC, MPL-2.0 (com cuidado). Qualquer outra exige aprovação humana.

Versões: use a versão estável mais recente de cada item no momento de iniciar a Fase 0, registre-a em docs/dependencias.md e não atualize sem tarefa específica.

### 6.1 Rust (host, viewer, relay, núcleo compartilhado)

- Linguagem: Rust stable, edição 2021 ou 2024, fixada em rust-toolchain.toml.
- Workspace: Cargo workspace na raiz, com crates core, host, viewer, relay e spikes (estes últimos sem publicação).
- Runtime assíncrono: tokio, multi-thread. Usado pelo iroh e pela rede.
- Rede e transporte:
  - iroh: conexões ponto a ponto, descoberta de endereço, atravessamento de NAT e fallback para relay. Use a API pública documentada em docs.rs, sem depender de detalhes internos.
  - quinn: implementação QUIC subjacente ao iroh. Acesse diretamente só se a tarefa exigir.
- Captura de tela: crate windows, com features Win32_Graphics_Dxgi, Win32_Graphics_Direct3D11 e Win32_Graphics_Dxgi_Common, usando DXGI Desktop Duplication (IDXGIOutputDuplication). Fallback: GDI BitBlt apenas se DXGI falhar, com aviso nos logs.
- Compressão: zstd, nível 1 para vídeo (prioridade a latência), nível 3 para arquivos.
- Serialização: postcard (binário compacto), serde para configuração e API.
- Injeção de entrada: crate windows, função SendInput, com KEYEVENTF_SCANCODE para teclado e coordenadas absolutas normalizadas para mouse.
- Serviço Windows: crate windows-service.
- Acesso a DPI: manifesto do aplicativo com per-monitor DPI awareness v2, configurado no build.
- Criptografia e segurança:
  - ed25519-dalek: identidade do PDV e assinatura de atualizações;
  - hmac e sha2: código de sessão e hash de arquivos (SHA-256);
  - subtle: comparação em tempo constante;
  - rand com OsRng: geração de códigos e nonces;
  - zeroize: apagar chaves e códigos da memória ao descartá-los.
- Proteção de chave no disco: DPAPI via crate windows (CryptProtectData, escopo de máquina).
- Interface do viewer: eframe e egui. Sem dependência de navegador ou runtime externo.
- Configuração: toml.
- Logs: tracing e tracing-subscriber, com rotação diária via tracing-appender, retenção de 30 dias.
- Erros: thiserror nas bibliotecas, anyhow apenas em binários.
- Testes: cargo test, proptest para protocolo, cargo-fuzz para decodificadores, criterion para benchmarks de captura e compressão.
- Auditoria de código: cargo clippy com -D warnings, cargo fmt, cargo audit e cargo deny.

### 6.2 Go (sinalização e API)

- Linguagem: Go, versão estável mais recente, fixada em go.mod.
- Roteamento: net/http da biblioteca padrão com chi.
- Banco: modernc.org/sqlite (sem CGO), arquivo local em /var/lib/prysmview/prysmview.db, permissões 0600.
- Logs: log/slog da biblioteca padrão, em JSON, sem dados sensíveis.
- Tokens de operador: opacos, aleatórios com 32 bytes, armazenados apenas como hash SHA-256 no banco, validade de 8 horas.
- Validação de assinatura do host: crypto/ed25519 da biblioteca padrão.
- Limite de requisições: implementado em memória com golang.org/x/time/rate, por IP e por token.
- Auditoria: govulncheck, gosec, go vet, gofmt.

### 6.3 Infraestrutura (custo zero)

- VM: Oracle Cloud Always Free, ARM Ampere A1, Ubuntu Server 24.04 LTS.
- Serviços: systemd para prysmview-api e prysmview-relay, com Restart=on-failure e usuário dedicado sem login.
- Firewall: ufw liberando apenas as portas necessárias (SSH restrito por chave, 443 TCP para API, porta do relay conforme configuração do iroh).
- Certificados: Let's Encrypt via Certbot, renovação automática por timer systemd. Domínio gratuito (DuckDNS) se necessário.
- Backup: script diário que copia o SQLite e o envia para um repositório privado no GitHub, com rotação de 30 dias.
- Monitoramento: UptimeRobot (plano gratuito) sobre o endpoint /health.

### 6.4 CI/CD e distribuição

- GitHub Actions (repositório público, uso ilimitado).
- Workflows: rust.yml (fmt, clippy, test, audit, deny), go.yml (gofmt, vet, govulncheck, gosec), release.yml (build, instalador, assinatura Ed25519, publicação do SHA-256).
- Instalador: WiX Toolset v4 ou Inno Setup 6, ambos gratuitos. Escolha única registrada em ADR.
- Atualização: o host consulta a última release no GitHub Releases, baixa o pacote, verifica assinatura Ed25519 contra a chave pública embutida no binário, e só então substitui os arquivos. Falha ao iniciar a nova versão: rollback automático para a anterior.
- Assinatura de código do Windows: não existe custo zero garantido. Enquanto não houver, documente o aviso do SmartScreen e forneça o SHA-256 publicado.

## 7. Estrutura do repositório

prysmview/
├── Cargo.toml workspace Rust
├── rust-toolchain.toml
├── AGENTS.md regras para agentes
├── LICENSE
├── README.md
├── crates/
│ ├── core/ protocolo, tipos, validações (sem I/O)
│ ├── host/ prysmview-svc (serviço) e lógica do host
│ ├── tray/ prysmview-tray (bandeja e consentimento)
│ ├── viewer/ prysmview-viewer (egui)
│ ├── relay/ configuração e binário do relay
│ └── spikes/ experimentos descartáveis S1 a S4
├── server/ API de sinalização em Go
│ ├── go.mod
│ ├── cmd/api/main.go
│ └── internal/...
├── installer/ WiX ou Inno Setup
├── deploy/ systemd, ufw, scripts de backup
├── docs/
│ ├── PROJETO.md
│ ├── threat-model.md
│ ├── protocolo.md
│ ├── dependencias.md
│ ├── glossario.md
│ ├── testes-seguranca.md
│ ├── adr/
│ ├── spikes/
│ └── plan/tasks.yaml
└── .github/workflows/


## 8. Protocolo (docs/protocolo.md é a versão detalhada)

Cabeçalho de todo quadro:
- magic: 4 bytes, ASCII "PRSM";
- versão do protocolo: u16, big-endian;
- tipo de mensagem: u8;
- tamanho do payload: u32, big-endian.

Limites de tamanho (validados antes de alocar memória):
- controle: 64 KiB;
- vídeo (um frame): 8 MiB;
- entrada: 256 bytes;
- chat: 4 KiB;
- arquivo (bloco): 1 MiB.

Canais:
- stream 0 (bidirecional): controle, handshake, código de sessão, permissões, encerramento;
- stream unidirecional host->viewer: vídeo;
- stream unidirecional viewer->host: entrada;
- streams bidirecionais por transferência: arquivos;
- stream bidirecional: chat e clipboard.

Mensagens de entrada:
- MouseMove: x e y como u16 normalizados (0 a 65535) sobre o monitor selecionado;
- MouseButton: botão, estado (pressionado ou solto);
- Wheel: delta vertical e horizontal;
- Key: scan code (u16), flag de tecla estendida, estado (pressionado ou solto).
Teclas são enviadas por scan code, nunca por código virtual, para respeitar o layout ABNT2 do PDV.

Vídeo:
- tiles de 64x64 pixels;
- cada frame tem id, número de tiles alterados, índice de cada tile e dados comprimidos com zstd;
- keyframe completo a cada 5 segundos ou sob solicitação do viewer;
- no máximo um frame em trânsito por vez; se o canal estiver lento, frames intermediários são descartados (nunca acumulados);
- meta de 15 a 30 fps, reduzida automaticamente quando a rede piora.

Versionamento: se a versão do viewer for diferente da do host em mais de uma versão maior, a sessão é recusada com mensagem clara para ambos.

## 9. Requisitos de segurança (inegociáveis)

1. Toda comunicação entre peers é QUIC com TLS 1.3. Nunca criar criptografia própria.
2. Nenhuma sessão inicia sem consentimento visível no PDV, exceto para operadores já marcados como confiáveis naquele PDV pelo próprio usuário local.
3. Código de sessão: 8 dígitos, 120 segundos, 3 tentativas, HMAC-SHA256 com comparação em tempo constante.
4. Bloqueio progressivo após tentativas erradas na API: 5 falhas bloqueiam o IP por 15 minutos.
5. Chave privada do PDV nunca sai do disco protegido por DPAPI, nunca aparece em log, nunca vai para o servidor.
6. Limites de tamanho validados antes de qualquer alocação.
7. unsafe somente com comentário // SAFETY: e teste associado.
8. Proibido unwrap() e expect() em código de rede, entrada e arquivos.
9. Logs de auditoria registram: operador, PDV, horário de início e fim, motivo informado, resultado do consentimento e quantidade de bytes transferidos. Nunca registram tela, teclas, conteúdo de arquivos ou códigos.
10. Ao encerrar uma sessão por qualquer motivo, todas as teclas e modificadores são soltos e as chaves de sessão são zeradas.
11. A janela de consentimento e o indicador de sessão não podem ser ocultados pelo viewer.
12. O host recusa qualquer comando de entrada ou vídeo que chegue sem sessão autenticada.
13. Atualização só é aplicada com assinatura Ed25519 válida.
14. Transferência de arquivos: verificação SHA-256 no destino antes de renomear o arquivo para o nome final. Em caso de falha, o arquivo parcial é removido.
15. Clipboard e transferência de arquivos podem ser desativados pelo usuário local no PDV.

## 10. Requisitos não-funcionais (metas mensuráveis)

- Latência de entrada (clique até efeito visível), em rede de loja com 20 ms de RTT: abaixo de 150 ms.
- Captura de vídeo em tela 1920x1080 parada: CPU do serviço abaixo de 5%.
- Captura de vídeo em movimento moderado: CPU do serviço abaixo de 15%, em processador de 4 núcleos de geração recente.
- Memória do serviço: abaixo de 150 MB em operação normal.
- Banda: tela parada abaixo de 50 kbit/s; navegação de texto abaixo de 1 Mbit/s.
- Suporte a DPI de 100%, 125%, 150% e 175%.
- Suporte a teclado ABNT2 e US-International.
- Suporte a Windows 10 22H2 e Windows 11 23H2 ou mais recente.
- Tamanho do instalador: abaixo de 30 MB.
- Início do serviço após boot: abaixo de 10 segundos.
- Atualização automática sem intervenção do usuário local, ocorrendo apenas com o PDV ocioso.

## 11. Limitações técnicas conhecidas

- Tela de logon e Ctrl+Alt+Del: exigem o serviço SYSTEM, troca de desktop para Winlogon e a política "Secure Attention Sequence" habilitada. Tratar na Fase 9, com teste em máquina real.
- Modo protegido do UAC: a captura de janelas elevadas exige a troca de desktop. Documentar se não for tratado na versão 1.
- DXGI Desktop Duplication pode ser perdido ao trocar de desktop ou mudar resolução. O código deve recriar a duplicação sem derrubar a sessão.
- Ferramentas de antivírus podem bloquear injeção de entrada. Documentar liberação para o time de TI.
- Windows 10 deixou de receber atualizações de segurança gratuitas em outubro de 2025. O software deve registrar a versão do SO no log, e a documentação deve alertar sobre a necessidade de ESU ou migração.

## 12. Ordem de construção (siga exatamente esta sequência)

Cada etapa só começa quando a anterior tem todos os critérios atendidos. Nenhuma etapa pode ser pulada.

### Fase 0: fundação
Entradas: nada.
Entregas:
1. Repositório com a estrutura da seção 7.
2. CI rust.yml e go.yml passando com crates e módulo de exemplo.
3. docs/PROJETO.md, docs/threat-model.md, AGENTS.md, docs/glossario.md, docs/dependencias.md.
4. ADR 0001: transporte (iroh/QUIC) e alternativas (WebRTC, protocolo próprio sobre TCP/TLS).
5. ADR 0002: escolha do instalador (WiX ou Inno Setup).
6. ADR 0003: código de sessão de 8 dígitos e modelo de consentimento.
Critério de pronto: CI verde, documentos aprovados pelo humano, dependências registradas com licença.

### Fase 1: spikes de viabilidade
Entradas: Fase 0 concluída.
Entregas, cada uma em crates/spikes com relatório em docs/spikes/:
- S1 captura DXGI: captura contínua de um monitor, medindo FPS e CPU. Meta: 30 fps em 1080p com CPU abaixo de 10%.
- S2 compressão de tiles: divisão em tiles 64x64, detecção de alterações e zstd. Meta: redução de banda acima de 90% em tela parada.
- S3 conexão iroh: dois processos em redes diferentes conectando via iroh, com registro se a conexão foi direta ou via relay.
- S4 injeção SendInput: clique e digitação ABNT2 por scan code em área de trabalho comum.
Critério de pronto: quatro relatórios com números medidos. Qualquer meta não atingida volta para revisão de arquitetura antes da Fase 2.

### Fase 2: protocolo
Entradas: Fase 1 aprovada.
Entregas:
1. crates/core com tipos de mensagem, cabeçalho e validação de limites.
2. Codificação e decodificação com postcard.
3. Testes proptest de ida e volta para todas as mensagens.
4. Fuzzing do decodificador por 30 minutos sem falhas.
5. docs/protocolo.md completo.
Critério de pronto: testes e fuzzing passam, documento aprovado.

### Fase 3: núcleo do host (captura e vídeo)
Entradas: Fase 2.
Entregas:
1. Captura DXGI em thread dedicada, com recriação automática da duplicação.
2. Detecção de tiles alterados e envio com zstd.
3. Keyframe periódico e sob demanda.
4. Controle de taxa com descarte de frames intermediários.
5. Seleção de monitor.
Critério de pronto: viewer de teste recebe tela real com CPU e latência dentro das metas da seção 10.

### Fase 4: injeção de entrada
Entradas: Fase 3.
Entregas:
1. Mapeamento de coordenadas normalizadas para o monitor selecionado, considerando DPI.
2. Injeção por scan code com KEYEVENTF_SCANCODE.
3. Liberação de todas as teclas e modificadores ao encerrar sessão ou perder conexão.
4. Teste com teclado ABNT2.
Critério de pronto: nenhuma tecla fica presa em nenhum cenário de teste de queda.

### Fase 5: sinalização e identidade (Go)
Entradas: Fase 2 (formato de mensagens de sinalização definido).
Entregas:
1. API com endpoints: registro do PDV, login do operador, lista de PDVs permitidos, solicitação de sessão, status de PDV.
2. Autenticação de requisições do host por assinatura Ed25519 com nonce e timestamp.
3. Tokens de operador opacos com hash no banco.
4. Limite de requisições por IP e por token.
5. Registro de auditoria de sessão.
6. Banco SQLite com migrações versionadas.
Critério de pronto: testes de integração cobrindo cada endpoint, incluindo requisição com assinatura inválida e token expirado, ambas recusadas.

### Fase 6: relay e conectividade
Entradas: Fase 5 e S3.
Entregas:
1. Relay iroh rodando como serviço systemd na VM.
2. Fallback automático: conexão direta, depois relay.
3. Testes com NAT simétrico e com firewall que bloqueia UDP.
4. Registro da proporção de sessões via relay.
Critério de pronto: conexão estabelecida em todos os cenários de teste.

### Fase 7: segurança de sessão
Entradas: Fase 5 e Fase 6.
Entregas:
1. Janela de consentimento na bandeja, com tempo limite de 60 segundos.
2. Código de sessão de 8 dígitos com validade e limite de tentativas.
3. Lista de operadores confiáveis definida pelo usuário local.
4. Indicador de sessão e botão de encerramento.
5. Pipe local com ACL restrita.
Critério de pronto: testes de invasão cobrindo código repetido, sessão sem consentimento, código expirado e tentativa de encerrar sessão a partir do viewer sem permissão. Todos devem ser bloqueados e registrados.

### Fase 8: viewer
Entradas: Fase 3, 4 e 7.
Entregas:
1. Lista de lojas e PDVs com status e busca.
2. Janela de sessão com escala, qualidade, tela cheia e seleção de monitor.
3. Renderização por tile, sem redesenhar a tela inteira.
4. Indicadores de latência e FPS.
Critério de pronto: operador sem treinamento consegue conectar e controlar o PDV em teste.

### Fase 9: recursos de sessão
Entradas: Fase 8.
Entregas:
1. Transferência de arquivos em blocos de 1 MiB, com SHA-256 e arquivo temporário.
2. Clipboard de texto com limite de 1 MB, desativável no PDV.
3. Chat de texto.
4. Ctrl+Alt+Del com serviço SYSTEM e troca de desktop, documentado conforme seção 11.
5. Reinício do PDV com reconexão automática.
Critério de pronto: arquivo chega íntegro com hash conferido, e Ctrl+Alt+Del funciona em máquina real.

### Fase 10: serviço Windows e instalador
Entradas: Fase 9.
Entregas:
1. Serviço prysmview-svc com inicialização automática e recuperação em falha.
2. prysmview-tray na sessão do usuário.
3. Instalador com WiX ou Inno Setup: copia arquivos, registra serviço, gera chave do dispositivo, cria atalhos.
4. Desinstalação limpa com confirmação.
5. Permissões mínimas nas pastas de programa.
Critério de pronto: instalação e remoção limpas em Windows 10 e Windows 11 sem intervenção manual.

### Fase 11: atualização assinada
Entradas: Fase 10.
Entregas:
1. Workflow release.yml gerando pacote, assinatura Ed25519 e SHA-256.
2. Verificação de assinatura antes da substituição.
3. Rollback automático se a nova versão falhar ao iniciar.
4. Canais estável e teste.
Critério de pronto: atualização de teste aplicada com sucesso em um PDV, e atualização com assinatura inválida recusada.

### Fase 12: testes e endurecimento
Entradas: Fase 11.
Entregas:
1. cargo audit, cargo deny, govulncheck e gosec sem achados altos.
2. Fuzzing contínuo de protocolo e configuração.
3. Testes de carga: vários viewers e hosts simultâneos na VM.
4. Testes de falha: queda de rede no meio da sessão, reinício do PDV, relay indisponível, disco cheio.
5. Revisão de segurança item a item contra o threat model.
6. Testes em Windows 10 e 11, com DPI 100%, 125%, 150% e 175%.
Critério de pronto: todos os testes passam e a revisão não tem itens abertos de severidade alta.

### Fase 13: piloto e operação
Entradas: Fase 12.
Entregas:
1. Piloto em uma ou duas lojas, com acompanhamento diário nas primeiras duas semanas.
2. Runbook: reinício do servidor, restauração do SQLite, rotação de chaves, revogação de PDV.
3. Backup diário testado com restauração real.
4. Monitoramento externo ativo.
5. Documentação para o time de TI da loja: instalação, liberação no antivírus, uso e resposta a incidentes.
6. Changelog e versionamento semântico.
Critério de pronto: piloto sem incidente crítico no período combinado, e simulação de falha do servidor resolvida dentro do runbook.

## 13. Convenções de código e processo

- Um agente trabalha somente nos arquivos listados em arquivos_permitidos da sua tarefa.
- Antes de codificar, descreva em até 10 linhas: o que vai fazer, arquivos afetados, riscos.
- Tarefas com mais de 5 arquivos ou 300 linhas de diff são divididas antes de começar.
- Commits pequenos no formato Conventional Commits (feat, fix, test, docs, refactor, chore).
- Todo módulo novo tem testes. Toda função de segurança tem teste de caso negativo.
- Nenhuma alteração de protocolo sem atualização de docs/protocolo.md e aprovação humana.
- Nenhuma dependência nova sem entrada em docs/dependencias.md e aprovação humana quando tocar segurança, criptografia ou rede.
- Nomes em código em inglês. Mensagens ao usuário final e documentação para a loja em português do Brasil.

## 14. Quando parar e perguntar

Pare, descreva o problema e aguarde decisão humana quando:
- a tarefa conflitar com a seção 9 (segurança);
- uma API de biblioteca não estiver clara na documentação oficial (docs.rs ou documentação do projeto), em vez de adivinhar;
- um teste de segurança falhar e a causa não estiver clara após duas tentativas;
- uma meta da seção 10 não for atingida na fase;
- for necessária uma dependência com licença fora da lista aceita.

Nunca contorne uma regra de segurança para fazer um teste passar.

## 15. Critério geral de conclusão do projeto

O PrysmView está concluído quando:
- todas as fases da seção 12 têm critério de pronto atendido e registrado;
- a revisão de segurança da Fase 12 não tem achados abertos de severidade alta;
- o piloto da Fase 13 foi concluído sem incidente crítico;
- a documentação do time de TI está publicada no repositório.
