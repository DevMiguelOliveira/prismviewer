# PrysmView

<p align="center">
  <img src="assets/logo.png" alt="PrysmView Logo" width="180" />
</p>

<p align="center">
  <strong>Software de Acesso Remoto Desktop de Alta Performance para PDVs (Caixas de Loja)</strong><br />
  <em>Windows 10 e Windows 11 • Latência Mínima • Criptografia Fim-a-Fim (QUIC TLS 1.3) • Custo Zero (R$ 0,00)</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2021%20Stable-orange?logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/Go-1.22+-blue?logo=go" alt="Go" />
  <img src="https://img.shields.io/badge/Protocol-QUIC%20%7C%20TLS%201.3-brightgreen" alt="Protocol" />
  <img src="https://img.shields.io/badge/Design-Cyber%20Prism%20UI-cyan" alt="Cyber Prism" />
  <img src="https://img.shields.io/badge/Tests-107%20Rust%20%7C%2014%20Go%20Passed-success" alt="Tests" />
  <img src="https://img.shields.io/badge/License-MIT-blue" alt="License" />
</p>

---

## 🎨 Identidade Visual Cyber Prism

O PrysmView utiliza uma linguagem visual inspirada no seu logotipo oficial prismático futurista:
- **Obsidian Dark Background:** `#0A0D16` / `#0F1422` (foco total e conforto visual para operadores).
- **Prism Neon Cyan:** `#00F0FF` (destaques primários, código de 8 dígitos e seleções ativas).
- **Electric Blue:** `#008CFF` (botões de ação e gradientes energéticos).
- **Prism Neon Violet:** `#BE2DEB` (badges do operador/técnico e prazos temporais).
- **Signal Mint Glow:** `#00FFCC` (indicador de conexão segura E2E e status online).
- **Emergency Crimson:** `#FF3355` (botão de encerramento imediato de acesso pelo caixa).

---

## 📦 Download do Instalador Oficial

- **Executável Setup Standalone:** [`dist/PrysmView-Setup-0.1.0.exe`](dist/PrysmView-Setup-0.1.0.exe)
- **Tamanho:** **5.19 MB** (limite NFR: < 30 MB)
- **SHA-256:** `77AA94B3F7942F15FC2F4A4927FCB529E4D265533DF52B8CDCF941252E12314F`
- **Conteúdo Incluso:** Serviço Windows `prysmview-svc.exe`, Aplicativo Host no PDV `prysmview-tray.exe`, Aplicativo do Técnico `prysmview-viewer.exe`, e inicialização no logon.

---

## 🌐 Infraestrutura Cloud Ativa (Custo Zero)

- **Servidor de Sinalização (API):** `http://20.121.137.44:443` (Azure Ampere A1 ARM64)
- **Relay P2P iroh (QUIC):** `20.121.137.44:3340` (UDP)
- **Health Check Público:** `http://20.121.137.44:443/health` (`{"status":"ok","version":"1.0.0"}`)

---

## 🏗️ Estrutura do Workspace

- `crates/core`: Protocolo, serialização compacta (`postcard`), limites de payload e regras sem I/O.
- `crates/host`: Serviço Windows (`prysmview-svc` em `LocalSystem`), captura DXGI, compressão zstd e injeção SendInput ABNT2.
- `crates/tray`: Aplicativo visual do PDV (`prysmview-tray`) estilo TeamViewer Host com travamento e código de sessão.
- `crates/viewer`: Aplicativo desktop do operador (`prysmview-viewer`) com login administrativo e canvas DXGI incremental.
- `crates/relay`: Binário e configuração do relay QUIC (`iroh-relay`).
- `crates/spikes`: Spikes de viabilidade técnica (S1: DXGI, S2: Tiles zstd, S3: iroh, S4: SendInput).
- `server/`: API de sinalização em Go com banco SQLite e autenticação de chaves Ed25519.

---

## 🚦 Execução dos Gates Determinísticos

Para garantir qualidade absoluta, nenhum código é aprovado por modelos de linguagem sem validação determinística:

### Gates Rust
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

### Gates Go (na pasta `server/`)
```bash
cd server
gofmt -l .
go vet ./...
go test ./...
```

---

## 🚀 Como Executar Localmente

### 1. Iniciar o Viewer (Operador / Técnico)
```bash
cargo run -p prysmview-viewer
```

### 2. Iniciar o Host em modo Console (PDV para testes)
```bash
cargo run -p prysmview-host --bin prysmview-svc -- --run
```

### 3. Iniciar a Bandeja do PDV (Host Interface)
```bash
cargo run -p prysmview-tray
```

### 4. Iniciar a API de Sinalização em Go
```bash
cd server
go run cmd/api/main.go
```

---

## 📚 Documentação do Projeto

- **Manual do Operador e PDV**: [docs/MANUAL-OPERADOR.md](docs/MANUAL-OPERADOR.md)
- **Atualização Assinada e Rollback**: [docs/atualizacao.md](docs/atualizacao.md)
- **Instalador Inno Setup 6**: [installer/README.md](installer/README.md)
- **Relatório de Segurança e Invasão**: [docs/testes-seguranca.md](docs/testes-seguranca.md)
- **Especificação do Protocolo**: [docs/protocolo.md](docs/protocolo.md)
- **Catálogo de Dependências**: [docs/dependencias.md](docs/dependencias.md)

