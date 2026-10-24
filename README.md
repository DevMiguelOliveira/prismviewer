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
