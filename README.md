# PrysmView - Acesso Remoto & Gestão de PDVs (React + Vite + Tailwind)

PrysmView é uma aplicação corporativa de acesso remoto desktop e gestão centralizada para PDVs (caixas de loja) rodando Windows 10 e Windows 11. O sistema foi reescrito em **React 18 + TypeScript + Vite + Tailwind CSS**, integrando a experiência do **Console do Operador (Admin Viewer)** e do **Terminal do Caixa (Host PDV)** em uma única interface fluida, rápida e responsiva.

## 🚀 Funcionalidades Portadas

### 1. Console do Operador (Admin Viewer)
- **Painel de Monitoramento (Dashboard)**:
  - Menu retrátil de Conexão Rápida (TeamViewer Style) com identificação do host local (`127.0.0.1`) e status de porta ativa (`19533`).
  - Indicadores e métricas de desempenho em tempo real: Total de PDVs, dispositivos Online/Offline e latência RTT em rede interna (18ms).
  - Catálogo de caixas de loja com filtros de status (Todos, Online, Offline), seletor de lojas/filiais e busca global.
  - Alternância entre visualização em Cards e visualização em Tabela detalhada.
  - Gaveta lateral (Drawer) com especificações de hardware (CPU, memória RAM, disco NVMe, resolução, SO e uptime).
  - Modal de cadastro de novos PDVs.
- **Conexão Rápida Direta**:
  - Conexão por ID ou endereço IP com código temporário de 8 dígitos ou senha de acesso desassistido (DPAPI).
  - Seleção de canais de conexão: Mouse/Teclado, Área de Transferência (Clipboard < 1MB) e Transferência de Arquivos (SHA-256).
- **Sessão Remota Interativa (In-Session Viewer)**:
  - Barra de ferramentas flutuante, retrátil e fixável (Pin/Collapse) inspirada no TeamViewer.
  - Telemetria viva: FPS (30–60), RTT (ms), kbps de vídeo e tempo de sessão decorrido.
  - Alternância em 1 clique entre **"Controle Total"** e **"Apenas Visualizar"** (suprime comandos remotos de mouse/teclado).
  - Escala adaptativa: **"Ajustar à Tela (Fit)"** e **"Tamanho Original (1:1)"**.
  - Envio de Sequência de Atenção Segura (**Ctrl+Alt+Del / SAS**) para desbloqueio ou logon no Windows.
  - Reinício remoto do PDV com alerta de reconexão automática.
  - Sincronização bidirecional de área de transferência (Clipboard).
  - Transferência de arquivos com verificação criptográfica SHA-256 no destino.
  - Chat de suporte em tempo real integrado entre o técnico e a operadora do caixa.
  - Simulador interativo de tela de PDV (Frente de Loja / Caixa) com cupom fiscal eletrônico, leitura de código de barras e abertura de gaveta via pulso elétrico serial (ESC/POS).
- **Grupos & Lojas**:
  - Gestão de unidades varejistas (Centro, Shopping Sul, Matriz, Filial Norte) com contatos, gerentes e contagem de caixas.
- **Auditoria E2E**:
  - Registro de sessões remotas com data/hora, operador, dispositivo, consentimento, volume transferido e assinatura imutável SHA-256.
  - Exportação de relatório completo em formato CSV.
- **Técnicos & Controle de Acesso (RBAC)**:
  - Gestão de equipe técnica (N1 Suporte Balcão, N2 Especialista Remoto, N3 Infraestrutura e Administrador TI).
- **Configurações Globais**:
  - Sinalização WebSocket/REST, servidores STUN/TURN (coturn Azure), timeouts e verificação de atualizações atômicas in-app.

### 2. Terminal do PDV (Host Mode)
- **Tela de Permissão de Suporte Remoto**:
  - "Sua ID" com cópia rápida para o clipboard.
  - Código temporário de 8 dígitos com temporizador regressivo de expiração (120 segundos) e botão para geração imediata de novo código.
  - Acesso desassistido com senha fixa configurável e opção de revelar/ocultar.
  - Travas rápidas de segurança para Área de Transferência e Arquivos.
- **Janela de Consentimento no PDV**:
  - Janela modal interativa com contagem regressiva de 60 segundos, exibindo nome do técnico, motivo informado e botões Aceitar/Recusar.
- **Banner de Sessão Ativa**:
  - Indicador pulsante e botão de emergência "Encerrar Acesso Imediatamente".

## 🛠️ Tecnologias Utilizadas

- **Framework**: React 18 + TypeScript 5
- **Build Tool**: Vite 5 (porta 3000 em `0.0.0.0`)
- **Estilização**: Tailwind CSS 3.4
- **Ícones**: Lucide React
- **Arquitetura**: Componentes modulares, estado reativo e design system escuro/claro

## 💻 Como Executar

```bash
# Instalação das dependências
npm install

# Inicialização do servidor de desenvolvimento na porta 3000
npm run dev

# Verificação de tipos
npm run lint

# Build de produção
npm run build
```
