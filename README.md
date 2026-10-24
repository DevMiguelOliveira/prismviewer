# PrysmView

Software de acesso remoto desktop de alta performance para PDVs (caixas de loja) rodando Windows 10 e Windows 11.
Projetado para latência mínima, baixo consumo de recursos, segurança de ponta a ponta e **custo zero** de infraestrutura.

---

## 🏗️ Estrutura do Workspace

- `crates/core`: Protocolo, serialização compacta (`postcard`), limites de payload e regras sem I/O.
- `crates/host`: Serviço Windows (`prysmview-svc` em `LocalSystem`), captura DXGI e injeção de entrada.
- `crates/tray`: Aplicativo da bandeja (`prysmview-tray`), consentimento e indicador de sessão no PDV.
- `crates/viewer`: Aplicativo desktop do operador (`prysmview-viewer`) baseado em `eframe`/`egui`.
- `crates/relay`: Binário e configuração do relay QUIC (`iroh-relay`).
- `crates/spikes`: Spikes de viabilidade técnica (S1: DXGI, S2: Tiles zstd, S3: iroh, S4: SendInput).
- `server/`: API de sinalização em Go com banco SQLite e autenticação Ed25519.

---

## 🚦 Execução dos Gates Determinísticos

Para garantir qualidade absoluta, nenhum código é aprovado por modelos de linguagem sem validação determinística:

### Gates Rust
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo audit
```

### Gates Go (na pasta `server/`)
```bash
cd server
gofmt -l .
go vet ./...
govulncheck ./...
gosec ./...
```

---

## 🔄 Orquestrador de Tarefas
O fluxo de desenvolvimento e verificação de worktrees é conduzido por:
```bash
python orchestrator.py
```
A fila e histórico de cada ciclo de tarefa residem em `docs/plan/tasks.yaml`.

---

## 🚀 Como Executar Localmente

### 1. Iniciar o Viewer (Operador)
```bash
cargo run -p prysmview-viewer
```

### 2. Iniciar o Host em modo Console (PDV para testes)
```bash
cargo run -p prysmview-host --bin prysmview-svc -- --run
```

### 3. Iniciar a Bandeja do PDV
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

