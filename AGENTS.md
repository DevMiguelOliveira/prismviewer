# AGENTS.md - PrysmView (React + Tailwind + Tauri)

## Regras de código e segurança

### Escopo de alteração
- O agente só pode alterar arquivos listados em `arquivos_permitidos` da tarefa em `docs/plan/tasks.yaml`.
- Qualquer alteração fora dessa lista é bloqueada pelo script de orquestração.
- Nenhuma tarefa começa com dependência aberta.

### Tamanho das tarefas
- No máximo 5 arquivos por tarefa.
- Cerca de 300 linhas de diff por tarefa.
- Acima disso, o script recusa e pede para dividir.

### Plano antes do código
- O agente descreve em até 10 linhas o que vai fazer, arquivos que vai alterar e riscos.
- Não escreva código até o plano ser aprovado pelo orquestrador.

### Tentativas
- Máximo de 3 ciclos com erro por tarefa.
- Após 3 tentativas falhas, a tarefa fica "bloqueada" e vai para o humano.

### Aprovação humana obrigatória
- Merge na main.
- Mudança em protocolo, criptografia ou segurança.
- Mudança em ADR.
- Dependência nova.
- Qualquer coisa tocando segurança.

### Sem auto-aprovação
- O agente que escreveu o código nunca é o que revisa.
- O revisor de segurança checa o diff contra o threat model.

### Linguagens suportadas
- TypeScript/JavaScript: ESLint, Prettier, TypeScript strict mode
- Rust (Tauri backend): cargo fmt --check, cargo clippy -- -D warnings, cargo test
- Go (sinalização): gofmt -l ., go vet ./..., govulncheck ./..., gosec ./...

# ESPECIFICAÇÃO COMPLETA: PRYSMVIEW

Você é um engenheiro sênior trabalhando no PrysmView. Este documento é a fonte de verdade do projeto. Se uma tarefa conflitar com ele, pare e pergunte. Não improvise arquitetura, protocolo, dependências ou comportamento de segurança.

## 0. Missão

PrysmView é um software de acesso remoto desktop para PDVs (caixas de loja) rodando Windows 10 e Windows 11. Um operador (técnico ou central de suporte) visualiza e controla o PDV de forma remota. O software deve ser:
- rápido: latência percebida baixa, fluidez de vídeo aceitável em rede de loja;
- leve: uso baixo de CPU, memória e disco no PDV, que costuma ter hardware modesto;
- seguro: criptografia fim a fim, consentimento visível no PDV, auditoria de sessões;
- de custo zero: nenhuma ferramenta, biblioteca, serviço, domínio ou certificado pago.

Interface visual inspirada no TeamViewer, em estilo limpo e funcional, construída com React + Tailwind CSS.

## 1. Atores

- Operador: pessoa que inicia a sessão a partir de outro computador (Admin Panel - viewer).
- Usuário da loja: pessoa que está no PDV. Deve sempre ver quando há acesso e pode recusá-lo ou encerrá-lo.
- Administrador: responsável por cadastrar PDVs, operadores e revogar acessos (via API e banco).

## 2. Fluxo de uma sessão (ordem exata)

1. O Host (app Tauri no PDV) inicia como serviço Windows e gera, na primeira execução, um par de chaves Ed25519. A chave pública é a identidade do PDV. A privada é protegida por DPAPI (escopo de máquina).
2. O Host se registra na sinalização com: chave pública, nome da loja, número do PDV, versão do agente e status.
3. O operador faz login na API de sinalização (token opaco de vida curta) no Admin Panel.
4. O operador escolhe um PDV. A API verifica se esse operador tem permissão de acesso a esse PDV.
5. A API informa ao Host, por canal de sinalização (WebSocket), que há uma solicitação de sessão, com id do operador e horário.
6. Se o operador não está na lista de confiança do PDV, o Host mostra uma janela de consentimento no PDV com nome do operador, motivo informado e botões Aceitar e Recusar. Sem resposta em 60 segundos, a sessão é recusada.
7. Ao aceitar, o Host gera um código de sessão de 8 dígitos, com validade de 120 segundos e no máximo 3 tentativas, e o exibe na própria janela do PDV. O operador digita esse código no Admin Panel. O código é verificado por HMAC no Host, com comparação em tempo constante.
8. O Admin Panel conecta-se ao Host por WebRTC (TURN/STUN), usando a chave pública do PDV como identidade.
9. Dentro da conexão, o primeiro canal é de controle. Nele ocorrem: handshake de versão, validação do código de sessão e definição de permissões (vídeo, entrada, arquivos, clipboard).
10. Os canais de vídeo, entrada, arquivos e chat são abertos conforme as permissões.
11. Durante a sessão, a janela de indicação no PDV fica visível e há um botão de encerramento imediato.
12. Ao encerrar (por qualquer lado, por queda ou por tempo máximo de 4 horas), o Host solta todas as teclas e modificadores, descarta chaves de sessão e grava o registro de auditoria.

## 3. Escopo

Incluído na versão 1:
- visualização da tela de um monitor por vez, com seleção de monitor;
- controle de mouse e teclado;
- transferência de arquivos com verificação de hash SHA-256;
- área de transferência compartilhada (texto, com limite de 1 MB), desativável;
- chat de texto simples;
- reinício do PDV com reconexão automática;
- Ctrl+Alt+Del a partir do Admin Panel;
- atualização automática assinada;
- auditoria de sessões.

Fora de escopo na versão 1 (não implementar, nem parcialmente):
- gravação de sessão em vídeo;
- acesso sem consentimento no PDV, em nenhuma hipótese;
- acesso a PDVs por navegador;
- áudio;
- múltiplas sessões simultâneas no mesmo PDV;
- gerenciamento de múltiplos monitores ao mesmo tempo (só um por vez);
- suporte a macOS e Linux no Host.

## 4. Premissas de confiança (threat model resumido)

- O PDV é um ativo valioso e pode estar em rede hostil (Wi-Fi compartilhado, rede da loja mal configurada).
- A rede entre operador e PDV é considerada hostil: qualquer pacote pode ser interceptado, alterado ou reenviado.
- O servidor de sinalização pode ser comprometido. Por isso ele nunca vê conteúdo de sessão (tela, teclas, arquivos), apenas metadados de conexão.
- O relay TURN vê apenas tráfego criptografado (DTLS), nunca o conteúdo.
- Um operador legítimo pode ter credencial vazada. Por isso há código de sessão, consentimento local e auditoria.
- O usuário local da loja pode tentar recusar ou encerrar acessos. O sistema deve respeitar essa decisão imediatamente.
- Um atacante com acesso físico ao PDV está fora de escopo.

## 5. Arquitetura de componentes

### Aplicações Desktop (Tauri)
- **prysmview-host**: App Tauri que roda no PDV. Interface React (Host UI) + backend Rust para captura, injeção de entrada, rede e criptografia. Instalado como serviço Windows (LocalSystem).
- **prysmview-admin**: App Tauri que roda no computador do operador. Interface React (Admin Panel) + backend Rust para comunicação P2P.

### Backend de Sinalização (Azure VM)
- **prysmview-api**: API REST + WebSocket em Go ou Node.js. Gerencia registro de hosts, autenticação de operadores, sinalização de sessões e auditoria.
- **prysmview-turn**: Servidor TURN/STUN (coturn) em Ubuntu na Azure VM, para atravessamento de NAT.

### Comunicação
- Host <-> Admin Panel: WebRTC (DTLS 1.3) com SRTP para mídia, SRTCP para controle. TURN server como fallback.
- Host <-> Sinalização: WebSocket seguro (WSS) com autenticação por assinatura Ed25519 de cada mensagem, usando nonce e timestamp para evitar replay.
- Host UI <-> Host Backend: IPC do Tauri (comandosinvoke).

## 6. Stack tecnológica

Regra geral: fixe versões no package-lock.json, Cargo.lock e go.sum. Toda dependência nova exige entrada em docs/dependencias.md com licença, motivo e alternativa considerada. Licenças aceitas: MIT, Apache-2.0, BSD-2/3, ISC, MPL-2.0 (com cuidado). Qualquer outra exige aprovação humana.

Versões: use a versão estável mais recente de cada item no momento de iniciar o projeto, registre-a em docs/dependencias.md e não atualize sem tarefa específica.

### 6.1 Frontend (React + Tailwind + Vite)

- **Framework**: React 18+ com TypeScript 5+
- **Build Tool**: Vite 5+
- **Styling**: Tailwind CSS 3.4+ com design system customizado
- **UI Components**: Radix UI (headless components) + customizados com Tailwind
- **State Management**: Zustand (lightweight) para estado global, React Context para local
- **Routing**: React Router v6+ (para navegação entre views)
- **Forms**: React Hook Form + Zod para validação
- **HTTP Client**: Axios ou Fetch API com interceptors
- **WebSocket**: nativo ou library leve (reconnecting-websocket)
- **Icons**: Lucide React (leve e extensível)
- **Icons**: Lucide React ou Heroicons
- **Linter**: ESLint com configuração strict
- **Formatter**: Prettier
- **Testing**: Vitest + React Testing Library
- **Type Safety**: TypeScript strict mode enabled

### 6.2 Tauri Backend (Rust)

- **Framework**: Tauri 2.0+ (latest stable)
- **Runtime assíncrono**: tokio, multi-thread
- **Captura de tela**: crate windows-rs com DXGI Desktop Duplication ou screencapture-ffi. Fallback: GDI BitBlt.
- **Compressão**: zstd, nível 1 para vídeo (prioridade a latência), nível 3 para arquivos.
- **Serialização**: serde + bincode (binário compacto)
- **Injeção de entrada**: crate windows-rs, função SendInput, com KEYEVENTF_SCANCODE para teclado.
- **Serviço Windows**: crate windows-service (via Rust backend do Tauri).
- **Criptografia e segurança**:
  - ed25519-dalek: identidade do PDV e assinatura de atualizações;
  - hmac e sha2: código de sessão e hash de arquivos (SHA-256);
  - subtle: comparação em tempo constante;
  - rand com OsRng: geração de códigos e nonces;
  - zeroize: apagar chaves e códigos da memória ao descartá-los.
- **Proteção de chave no disco**: DPAPI via crate windows-rs (CryptProtectData, escopo de máquina).
- **Configuração**: toml ou JSON via Tauri API.
- **Logs**: tracing e tracing-subscriber, com rotação diária via tracing-appender, retenção de 30 dias.
- **Erros**: thiserror e anyhow.
- **Auditoria de código**: cargo clippy com -D warnings, cargo fmt, cargo audit e cargo deny.

### 6.3 Backend de Sinalização (Go ou Node.js)

**Opção A: Go (recomendado para performance)**
- Linguagem: Go 1.21+
- Roteamento: net/http da biblioteca padrão com chi ou gin
- WebSocket: gorilla/websocket
- Banco: modernc.org/sqlite (sem CGO) ou PostgreSQL se necessário
- Logs: log/slog da biblioteca padrão, em JSON, sem dados sensíveis
- Tokens de operador: opacos, aleatórios com 32 bytes, armazenados apenas como hash SHA-256
- Validação de assinatura: crypto/ed25519 da biblioteca padrão
- Limite de requisições: golang.org/x/time/rate
- Auditoria: govulncheck, gosec, go vet, gofmt

**Opção B: Node.js (mais alinhado com stack frontend)**
- Runtime: Node.js 20 LTS
- Framework: Fastify ou Express.js
- WebSocket: ws ou Socket.io
- Banco: SQLite (better-sqlite3) ou PostgreSQL
- Logs: pino ou winston
- Validação: Joi ou Zod
- Linter: ESLint + Prettier
- Auditoria: npm audit, Snyk

### 6.4 Infraestrutura (Azure VM - custo zero com créditos)

- **VM**: Azure B1s burstable (1 vCPU, 1GB RAM) ou similar, Ubuntu Server 22.04 LTS
- **Serviços**: systemd para prysmview-api e prysmview-turn, com Restart=on-failure
- **Firewall**: ufw liberando apenas as portas necessárias (SSH restrito por chave, 443 TCP para API, 3478/5349 para TURN)
- **Certificados**: Let's Encrypt via Certbot, renovação automática por timer systemd. Domínio gratuito (DuckDNS ou Freenom se disponível)
- **Backup**: script diário que copia o SQLite e o envia para um repositório privado no GitHub, com rotação de 30 dias
- **Monitoramento**: UptimeRobot (plano gratuito) sobre o endpoint /health

### 6.5 CI/CD e distribuição

- **GitHub Actions** (repositório público, uso ilimitado)
- **Workflows**:
  - frontend.yml: ESLint, Prettier, TypeScript check, Vitest
  - tauri.yml: cargo fmt, cargo clippy, cargo test, cargo audit
  - backend.yml: gofmt, go vet, govulncheck, gosec (ou equivalentes Node.js)
  - release.yml: build Tauri, assinatura Ed25519, publicação do SHA-256
- **Instalador**: Tauri gera automaticamente MSI/EXE para Windows
- **Atualização**: o Host consulta a última release no GitHub Releases via Tauri Updater, baixa o pacote, verifica assinatura Ed25519 contra a chave pública embutida no binário, e só então substitui os arquivos. Falha ao iniciar a nova versão: rollback automático para a anterior.
- **Assinatura de código do Windows**: não existe custo zero garantido. Enquanto não houver, documente o aviso do SmartScreen e forneça o SHA-256 publicado.

## 7. Estrutura do repositório

prysmview/
├── package.json (root workspace)
├── pnpm-workspace.yaml
├── turbo.json (build orchestration)
├── LICENSE
├── README.md
├── AGENTS.md regras para agentes
├── apps/
│ ├── host/ (Tauri app - Host UI + Backend)
│ │ ├── src-tauri/ (Rust backend)
│ │ │ ├── Cargo.toml
│ │ │ ├── src/
│ │ │ └── tauri.conf.json
│ │ ├── src/ (React frontend)
│ │ │ ├── components/
│ │ │ ├── pages/
│ │ │ ├── stores/
│ │ │ ├── lib/
│ │ │ ├── App.tsx
│ │ │ └── main.tsx
│ │ ├── package.json
│ │ ├── vite.config.ts
│ │ ├── tailwind.config.js
│ │ └── tsconfig.json
│ └── admin/ (Tauri app - Admin Panel)
│ ├── src-tauri/ (Rust backend)
│ │ ├── Cargo.toml
│ │ ├── src/
│ │ └── tauri.conf.json
│ ├── src/ (React frontend)
│ │ ├── components/
│ │ ├── pages/
│ │ ├── stores/
│ │ ├── lib/
│ │ ├── App.tsx
│ │ └── main.tsx
│ ├── package.json
│ ├── vite.config.ts
│ ├── tailwind.config.js
│ └── tsconfig.json
├── packages/
│ ├── ui/ (Componentes compartilhados)
│ │ ├── src/
│ │ │ ├── components/
│ │ │ │ ├── Button/
│ │ │ │ ├── Input/
│ │ │ │ ├── Tabs/
│ │ │ │ └── Modal/
│ │ │ ├── lib/
│ │ │ └── index.ts
│ │ ├── package.json
│ │ └── tsconfig.json
│ ├── config/ (Configurações compartilhadas)
│ │ ├── tailwind.config.js
│ │ ├── tsconfig.base.json
│ │ └── eslint-config.js
│ └── types/ (Tipos TypeScript compartilhados)
│ ├── src/
│ │ ├── protocol.ts
│ │ ├── session.ts
│ │ └── index.ts
│ └── package.json
├── server/ (Backend de sinalização)
│ ├── go.mod (ou package.json se Node.js)
│ ├── cmd/api/main.go
│ ├── internal/
│ └── deployments/
├── docs/
│ ├── PROJETO.md
│ ├── threat-model.md
│ ├── protocolo.md
│ ├── dependencias.md
│ ├── glossario.md
│ ├── testes-seguranca.md
│ ├── adr/
│ └── plan/tasks.yaml
└── .github/workflows/

## 8. Protocolo (docs/protocolo.md é a versão detalhada)

Cabeçalho de todo quadro (via WebRTC data channel):
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

Canais WebRTC:
- data channel 0 (bidirecional): controle, handshake, código de sessão, permissões, encerramento;
- data channel unidirecional host->viewer: vídeo;
- data channel unidirecional viewer->host: entrada;
- data channels bidirecionais por transferência: arquivos;
- data channel bidirecional: chat e clipboard.

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

1. Toda comunicação entre peers é WebRTC com DTLS 1.3. Nunca criar criptografia própria.
2. Nenhuma sessão inicia sem consentimento visível no PDV, exceto para operadores já marcados como confiáveis naquele PDV pelo próprio usuário local.
3. Código de sessão: 8 dígitos, 120 segundos, 3 tentativas, HMAC-SHA256 com comparação em tempo constante.
4. Bloqueio progressivo após tentativas erradas na API: 5 falhas bloqueiam o IP por 15 minutos.
5. Chave privada do PDV nunca sai do disco protegido por DPAPI, nunca aparece em log, nunca vai para o servidor.
6. Limites de tamanho validados antes de qualquer alocação.
7. unsafe no Rust somente com comentário // SAFETY: e teste associado.
8. Proibido unwrap() e expect() em código de rede, entrada e arquivos.
9. Logs de auditoria registram: operador, PDV, horário de início e fim, motivo informado, resultado do consentimento e quantidade de bytes transferidos. Nunca registram tela, teclas, conteúdo de arquivos ou códigos.
10. Ao encerrar uma sessão por qualquer motivo, todas as teclas e modificadores são soltos e as chaves de sessão são zeradas.
11. A janela de consentimento e o indicador de sessão não podem ser ocultados pelo viewer.
12. O host recusa qualquer comando de entrada ou vídeo que chegue sem sessão autenticada.
13. Atualização só é aplicada com assinatura Ed25519 válida.
14. Transferência de arquivos: verificação SHA-256 no destino antes de renomear o arquivo para o nome final. Em caso de falha, o arquivo parcial é removido.
15. Clipboard e transferência de arquivos podem ser desativados pelo usuário local no PDV.
16. Atualização in-app no Windows (NTFS): substituição atômica in-place através de renomeação temporária (.old), com verificação prévia obrigatória de formato PE (magic bytes `b"MZ"` e tamanho > 100 KB) e SHA-256 em tempo constante antes de qualquer escrita física. Proibida simulação fictícia via timers (setInterval/setTimeout) na UI; o progresso deve ser derivado de eventos reais do backend nativo.
17. Telemetria e conectividade real (Zero Mocks): nenhum dispositivo é presumido online sem validação de conectividade ativa via socket e porta de serviço (19533).
18. Ergonomia e UI/UX sem Redundâncias: No Host, foco primário nas credenciais de acesso rápido (Sua ID + Código de 8 dígitos) e isolamento de Acesso Desassistido (Senha Fixa). No Admin, proibição de botões duplicados de ação e elementos mortos ocultos; a interface deve possuir regras completas de responsividade fluida via CSS (@media) com sidebar colapsável em telas compactas (< 1150px) e Drawer como overlay lateral flutuante.
19. Ergonomia e Controles In-Session Inspirados no TeamViewer (Custo Zero):
    - Barra Superior Retrátil (Toolbar Pin/Collapse): A barra de ferramentas da sessão remota no Viewer deve ser flutuante, centralizada e retrátil, permitindo recolhimento suave para o topo da tela sem obstruir menus ou cabeçalhos do PDV.
    - Modo Apenas Visualizar (View-Only / Block Remote Input): O Viewer deve oferecer alternância imediata em 1 clique entre "Controle Total" e "Apenas Visualizar", suprimindo eventos de mouse e teclado quando o operador estiver apenas observando o caixa de loja trabalhar.
    - Escala Adaptativa: Suporte a modos "Ajustar à Tela (Fit)" e "Tamanho Original 1:1" com rolagem fluida.
    - Telemetria e Qualidade em Tempo Real: Exibição viva de FPS, RTT (ms) e kbps com alternância entre perfis "Fluidez (60 FPS)" e "Alta Resolução (HD)".

## 10. Requisitos não-funcionais (metas mensuráveis)

- Latência de entrada (clique até efeito visível), em rede de loja com 20 ms de RTT: abaixo de 150 ms.
- Captura de vídeo em tela 1920x1080 parada: CPU do serviço abaixo de 5%.
- Captura de vídeo em movimento moderado: CPU do serviço abaixo de 15%, em processador de 4 núcleos de geração recente.
- Memória do serviço: abaixo de 150 MB em operação normal.
- Banda: tela parada abaixo de 50 kbit/s; navegação de texto abaixo de 1 Mbit/s.
- Suporte a DPI de 100%, 125%, 150% e 175%.
- Suporte a teclado ABNT2 e US-International.
- Suporte a Windows 10 22H2 e Windows 11 23H2 ou mais recente.
- Tamanho do instalador: abaixo de 50 MB (Tauri ~30MB + app ~20MB).
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
1. Repositório com a estrutura da seção 7 (monorepo com pnpm workspace).
2. CI frontend.yml, tauri.yml e backend.yml passando com apps de exemplo.
3. docs/PROJETO.md, docs/threat-model.md, AGENTS.md, docs/glossario.md, docs/dependencias.md.
4. ADR 0001: transporte (WebRTC) e alternativas (QUIC/iroh, protocolo próprio).
5. ADR 0002: escolha do backend de sinalização (Go vs Node.js).
6. ADR 0003: código de sessão de 8 dígitos e modelo de consentimento.
Critério de pronto: CI verde, documentos aprovados pelo humano, dependências registradas com licença.

### Fase 1: spikes de viabilidade
Entradas: Fase 0 concluída.
Entregas:
1. Spike captura de tela: proof-of-concept em Rust usando DXGI Desktop Duplication.
2. Spike WebRTC: proof-of-concept de conexão P2P entre duas instâncias Tauri.
3. Spike injeção de entrada: proof-of-concept de SendInput em Rust.
4. Spike DPAPI: proof-of-concept de proteção de chave privada.
Critério de pronto: todos os spikes funcionando, documentados em docs/spikes/.

### Fase 2: design system e componentes compartilhados
Entradas: Fase 1 concluída.
Entregas:
1. Design system completo em Tailwind (cores, tipografia, espaçamento, componentes).
2. Pacote packages/ui com componentes base (Button, Input, Tabs, Modal, etc.).
3. Pacote packages/types com tipos TypeScript compartilhados.
4. Pacote packages/config com configurações compartilhadas (Tailwind, ESLint, TypeScript).
Critério de pronto: Storybook ou demo funcionando, todos os componentes testados.

### Fase 3: Host UI (React)
Entradas: Fase 2 concluída.
Entregas:
1. App Host com Tauri configurado.
2. Interface de consentimento (janela modal com botões Aceitar/Recusar).
3. Indicador de sessão ativa (tray icon + janela flutuante).
4. Configurações locais (confiança de operadores, permissões).
5. IPC commands para comunicação com backend Rust.
Critério de pronto: UI funcional, comunicação IPC testada, build Tauri funcionando.

### Fase 4: Admin Panel (React)
Entradas: Fase 2 concluída.
Entregas:
1. App Admin com Tauri configurado.
2. Dashboard de dispositivos (lista de PDVs online/offline).
3. Formulário de conexão (ID do parceiro + código de sessão).
4. Interface de sessão ativa (vídeo, controles, chat, transferência de arquivos).
5. Barra de ferramentas retrátil inspirada no TeamViewer.
6. Controles de qualidade (FPS, RTT, kbps, modo view-only).
Critério de pronto: UI funcional, navegação por tabs corrigida, botões funcionando, build Tauri funcionando.

### Fase 5: Backend de sinalização
Entradas: Fase 3 e Fase 4 concluídas.
Entregas:
1. API REST para registro de hosts e autenticação de operadores.
2. WebSocket para sinalização de sessões.
3. Banco de dados (SQLite) com schema completo.
4. Validação de assinaturas Ed25519.
5. Rate limiting e proteção contra abuse.
6. Logs de auditoria.
Critério de pronto: API funcionando, endpoints testados, integração com Host e Admin.

### Fase 6: Host Backend (Rust - Tauri)
Entradas: Fase 1 e Fase 3 concluídas.
Entregas:
1. Captura de tela com DXGI Desktop Duplication.
2. Compressão de vídeo com zstd.
3. Geração de par de chaves Ed25519 + proteção DPAPI.
4. Registro na API de sinalização.
5. WebSocket de sinalização.
6. Conexão WebRTC com Admin Panel.
7. Injeção de entrada (mouse e teclado).
8. Serviço Windows (instalação e inicialização).
Critério de pronto: Host conectando-se à API, capturando tela e aceitando conexões.

### Fase 7: Admin Backend (Rust - Tauri)
Entradas: Fase 1 e Fase 4 concluídas.
Entregas:
1. Login na API de sinalização.
2. Listagem de PDVs disponíveis.
3. Conexão WebRTC com Host.
4. Decodificação e renderização de vídeo.
5. Envio de eventos de entrada.
6. Chat e transferência de arquivos.
7. Controles de qualidade (ajuste de FPS, resolução).
Critério de pronto: Admin conectando-se a Host, exibindo vídeo e controlando remotamente.

### Fase 8: integração end-to-end
Entradas: Fase 5, Fase 6 e Fase 7 concluídas.
Entregas:
1. Sessão completa funcionando (do consentimento ao encerramento).
2. Auditoria de sessões registrada corretamente.
3. Atualização automática funcionando.
4. Testes de segurança (penetration test básico).
5. Testes de performance (latência, CPU, memória).
Critério de pronto: sessão end-to-end funcionando em rede real, métricas dentro dos limites.

### Fase 9: produção
Entradas: Fase 8 concluída.
Entregas:
1. Deploy em Azure VM (API + TURN).
2. Configuração de domínio + certificados Let's Encrypt.
3. Instaladores MSI/EXE gerados e testados.
4. Documentação de instalação e uso.
5. Script de backup automático.
6. Monitoramento (UptimeRobot).
Critério de pronto: sistema rodando em produção, acessível a operadores reais.

## 13. Convenções de código React + Tailwind

### React
- Use TypeScript strict mode.
- Componentes funcionais com hooks.
- Use useId() para IDs gerados.
- Use useCallback() para callbacks passados como props.
- Use useMemo() para cálculos pesados.
- Use custom hooks para lógica reutilizável.
- Props tipadas explicitamente.
- Evite any e unknown.
- Use Zod para validação de dados externos.

### Tailwind CSS
- Siga o design system definido em packages/config/tailwind.config.js.
- Use classes utilitárias, não escreva CSS customizado a menos que necessário.
- Use @apply apenas para componentes complexos reutilizáveis.
- Responsividade via @media: mobile-first.
- Cores: use variáveis CSS customizadas para tema claro/escuro.
- Espaçamento: use escala consistente (4px base).
- Tipografia: use escala consistente de tamanhos.

### Estado
- Estado local: useState.
- Estado global: Zustand.
- Estado de servidor: React Query ou SWR.
- Estado de formulário: React Hook Form + Zod.

### Navegação
- Use React Router v6+.
- Rotas protegidas via componente wrapper.
- Loading states e error boundaries.
- Histórico de navegação preservado.

### Tabs
- Implementação com componente customizado baseado em Radix UI Tabs.
- Estado controlado com activeTab.
- Animações suaves com framer-motion ou CSS transitions.
- Lazy loading de conteúdo quando apropriado.

### Botões
- Use componente Button do packages/ui.
- Variantes: primary, secondary, ghost, danger.
- Estados: default, hover, active, disabled, loading.
- Props: onClick, disabled, loading, icon, children.
- Acessibilidade: type="button", aria-label quando necessário.

## 14. Comandos úteis

### Desenvolvimento
```bash
# Instalar dependências
pnpm install

# Executar Host em modo dev
pnpm --filter host dev

# Executar Admin em modo dev
pnpm --filter admin dev

# Executar ambos em modo dev
pnpm dev

# Build de produção
pnpm build

# Build de instaladores Tauri
pnpm --filter host tauri build
pnpm --filter admin tauri build
```

### Linting e formatação
```bash
# Lint all
pnpm lint

# Lint específico
pnpm --filter host lint
pnpm --filter admin lint

# Format all
pnpm format

# Type check
pnpm typecheck
```

### Testes
```bash
# Executar todos os testes
pnpm test

# Testes específicos
pnpm --filter host test
pnpm --filter admin test

# Testes e2e
pnpm test:e2e
```

### Backend (Go)
```bash
# Executar API
cd server
go run cmd/api/main.go

# Testes
go test ./...

# Lint
gofmt -l .
go vet ./...
govulncheck ./...
gosec ./...
```

## 15. Checklist para pull requests

- [ ] Código segue as convenções definidas.
- [ ] TypeScript strict mode sem erros.
- [ ] ESLint sem warnings.
- [ ] Prettier aplicado.
- [ ] Testes unitários passando.
- [ ] Testes e2e passando (se aplicável).
- [ ] Dependências registradas em docs/dependencias.md.
- [ ] Licenças verificadas (MIT, Apache-2.0, BSD-2/3, ISC, MPL-2.0).
- [ ] Security review feito (para mudanças em protocolo/criptografia).
- [ ] Performance não degradada.
- [ ] Documentação atualizada (se necessário).
- [ ] AGENTS.md não violado.
