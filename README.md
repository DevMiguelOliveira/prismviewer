# PrysmView

Software de acesso remoto peer-to-peer open-source inspirado no TeamViewer.

## 🎯 Missão

PrysmView é um software de acesso remoto desktop para PDVs (caixas de loja) rodando Windows 10 e Windows 11. Um operador (técnico ou central de suporte) visualiza e controla o PDV de forma remota. O software deve ser:

- **rápido**: latência percebida baixa, fluidez de vídeo aceitável em rede de loja;
- **leve**: uso baixo de CPU, memória e disco no PDV, que costuma ter hardware modesto;
- **seguro**: criptografia fim a fim, consentimento visível no PDV, auditoria de sessões;
- **de custo zero**: nenhuma ferramenta, biblioteca, serviço, domínio ou certificado pago.

## 🏗️ Arquitetura

PrysmView v2.0 é construído com:

- **Frontend**: React 18 + TypeScript + Tailwind CSS + Vite
- **Desktop Framework**: Tauri 2.0 (Rust backend)
- **Comunicação P2P**: WebRTC (TURN/STUN)
- **Backend de Sinalização**: Go ou Node.js (Azure VM)
- **Design System**: Customizado com Tailwind CSS (inspirado no TeamViewer)

### Componentes

- **prysmview-host**: App Tauri que roda no PDV (Host UI + Backend Rust)
- **prysmview-admin**: App Tauri que roda no computador do operador (Admin Panel + Backend Rust)
- **@prysmview/ui**: Pacote de componentes compartilhados
- **prysmview-api**: API de sinalização (Go/Node.js)
- **prysmview-turn**: Servidor TURN/STRTUN (coturn)

## 🚀 Quick Start

### Pré-requisitos

- Node.js 18+
- pnpm 8+
- Rust 1.70+
- Windows 10/11 (para desenvolvimento e uso)

### Setup Inicial

```powershell
# Executar o script de setup
.\setup_prysmview.ps1
```

Este script vai:
1. Verificar dependências (Node.js, pnpm, Rust)
2. Criar estrutura de diretórios (monorepo com pnpm workspace)
3. Inicializar apps Host e Admin com Tauri
4. Criar pacote de componentes compartilhados
5. Instalar todas as dependências
6. Inicializar repositório Git

### Desenvolvimento

```powershell
# Executar Host em modo dev
pnpm --filter prysmview-host dev

# Executar Admin em modo dev
pnpm --filter prysmview-admin dev

# Executar ambos em modo dev
pnpm dev

# Linting
pnpm lint

# Type checking
pnpm typecheck

# Formatação
pnpm format

# Testes
pnpm test
```

### Build de Produção

```powershell
# Build de todos os apps
pnpm build

# Build de instaladores Tauri
.\build_prysmview.ps1
```

Os instaladores (.msi e .exe) serão gerados na pasta `Releases/`.

## 📁 Estrutura do Repositório

```
prysmview/
├── apps/
│ ├── host/ (Tauri app - Host UI + Backend)
│ └── admin/ (Tauri app - Admin Panel + Backend)
├── packages/
│ ├── ui/ (Componentes compartilhados)
│ ├── config/ (Configurações compartilhadas)
│ └── types/ (Tipos TypeScript compartilhados)
├── server/ (Backend de sinalização - Go/Node.js)
├── docs/
│ ├── PROJETO.md
│ ├── threat-model.md
│ ├── protocolo.md
│ ├── dependencias.md
│ ├── adr/
│ └── spikes/
├── AGENTS.md (Regras para agentes de IA)
├── setup_prysmview.ps1 (Script de setup)
├── build_prysmview.ps1 (Script de build)
└── README.md
```

## 🔒 Segurança

PrysmView segue um modelo de segurança rigoroso:

- Criptografia fim a fim (WebRTC com DTLS 1.3)
- Consentimento visível no PDV (nunca acesso sem permissão)
- Código de sessão de 8 dígitos (HMAC-SHA256)
- Auditoria completa de sessões
- Chaves privadas protegidas por DPAPI
- Nenhum dado sensível em logs

Veja [AGENTS.md](AGENTS.md) para requisitos de segurança detalhados.

## 📚 Documentação

- [AGENTS.md](AGENTS.md) - Regras para desenvolvimento e segurança
- [docs/PROJETO.md](docs/PROJETO.md) - Visão geral do projeto
- [docs/threat-model.md](docs/threat-model.md) - Modelo de ameaças
- [docs/protocolo.md](docs/protocolo.md) - Protocolo de comunicação
- [docs/dependencias.md](docs/dependencias.md) - Dependências e licenças
- [deploy/HTTPS-SETUP.md](deploy/HTTPS-SETUP.md) - Configuração HTTPS com Caddy
- [deploy/AZURE.md](deploy/AZURE.md) - Deploy na Azure VM

## 🤝 Contribuindo

Leia [AGENTS.md](AGENTS.md) antes de contribuir. Este projeto tem regras estritas de segurança e qualidade de código.

## 📄 Licença

MIT OR Apache-2.0

## 🙏 Agradecimentos

Inspirado no TeamViewer e nas melhores práticas de software de acesso remoto open-source.
