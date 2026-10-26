# Catálogo de Dependências - PrysmView

Este documento registra as dependências iniciais do projeto conforme especificado. Todas seguem licenças aceitas (MIT, Apache-2.0, BSD-2-Clause/BSD-3-Clause, ISC, MPL-2.0 com cautela). Novas dependências exigem entrada deste arquivo e, quando aplicável, aprovação humana.

## Rust (host, viewer, relay, núcleo compartilhado)

| Dependência | Uso no PrysmView | Licença | Motivo de escolha |
|-------------|-------------------|---------|-------------------|
| [`tokio`](https://crates.io/crates/tokio) | Runtime assíncrono multi-thread | MIT | Padrão da indústria, integra-se nativamente com iroh/QUIC e oferece eficiência para I/O assíncrono. |
| [`iroh`](https://crates.io/crates/iroh) | Transporte P2P QUIC/TLS 1.3, descoberta, NAT traversal e fallback para relay | Apache-2.0 / MIT | Atende diretamente aos requisitos de conexão ponto-a-ponto segura sem implementar QUIC do zero. |
| [`quinn`](https://crates.io/crates/quinn) | Implementação QUIC subjacente ao iroh | Apache-2.0 / MIT | Robusto, bem documentado e utilizado pelo ecossistema iroh. Acesso direto apenas se necessário. |
| [`zstd`](https://crates.io/crates/zstd) | Compressão de tiles de vídeo e blocos de arquivo | BSD-2-Clause | Excelente relação compressão/latência. Nível 1 ideal para vídeo em tempo real. |
| [`postcard`](https://crates.io/crates/postcard) | Serialização binária compacta do protocolo | Apache-2.0 / MIT | Leve, sem overhead desnecessário e ideal para mensagens binárias com limites de tamanho. |
| [`ed25519-dalek`](https://crates.io/crates/ed25519-dalek) | Identidade Ed25519 do PDV e assinatura de atualizações | BSD-3-Clause | Implementação moderna, segura e amplamente auditada para assinaturas Ed25519. |
| [`hmac`](https://crates.io/crates/hmac) | Geração/verificação do código de sessão | MIT / Apache-2.0 | Padrão para HMAC-SHA256, necessário para o código de sessão de 8 dígitos. |
| [`sha2`](https://crates.io/crates/sha2) | Hash SHA-256 (código, arquivos) | MIT / Apache-2.0 | Implementação confiável e padrão para verificação de integridade. |
| [`subtle`](https://crates.io/crates/subtle) | Comparação em tempo constante | BSD-3-Clause | Essencial para mitigar ataques por temporização em HMAC/códigos de sessão. |
| [`rand`](https://crates.io/crates/rand) | Geração criptograficamente segura com OsRng | MIT / Apache-2.0 | Usado para códigos de sessão, nonces e valores aleatórios seguros. |
| [`zeroize`](https://crates.io/crates/zeroize) | Zerar chaves, códigos e segredos da memória | Apache-2.0 / MIT | Fundamental para reduzir janela de exposição de segredos em memória. |
| [`windows-service`](https://crates.io/crates/windows-service) | Implementação do serviço Windows `prysmview-svc` | MIT / Apache-2.0 | Facilita integração correta com SCM do Windows (inicialização, recuperação, parada). |
| [`windows`](https://crates.io/crates/windows) | APIs Win32: DXGI, Direct3D11, SendInput, DPAPI, serviços | MIT / Apache-2.0 | Acesso nativo às APIs necessárias (captura DXGI, injeção de entrada, proteção DPAPI). |
| [`reqwest`](https://crates.io/crates/reqwest) | Cliente HTTP assíncrono para comunicação com a API Go | MIT / Apache-2.0 | Cliente HTTP padrão da comunidade com rustls-tls e suporte a JSON. |
| [`serde`](https://crates.io/crates/serde) | Framework de serialização e desserialização de dados | MIT / Apache-2.0 | Padrão do ecossistema Rust para JSON e outros formatos. |
| [`serde_json`](https://crates.io/crates/serde_json) | Serialização e desserialização JSON para APIs REST | MIT / Apache-2.0 | Padrão para JSON na comunicação HTTP com a API Go. |
| [`thiserror`](https://crates.io/crates/thiserror) | Tipagem estruturada e ergonômica de erros em bibliotecas | MIT / Apache-2.0 | Erros tipados em Rust sem alocações desnecessárias. |
| [`tracing`](https://crates.io/crates/tracing) | Instrumentação e logging estruturado de eventos | MIT | Padrão assíncrono de observabilidade e auditoria segura. |
| [`eframe`](https://crates.io/crates/eframe) | Framework GUI desktop baseado em egui | MIT / Apache-2.0 | Interface gráfica nativa e leve para a bandeja do PDV (`PrysmHost`). |
| [`tao`](https://crates.io/crates/tao) | Biblioteca de criação de janelas desktop (Tauri Core) | MIT / Apache-2.0 | Suporte a janelas nativas, DPI, eventos e barra de título no Windows 10/11. |
| [`wry`](https://crates.io/crates/wry) | Renderizador WebView2 multiplataforma (Tauri Core) | MIT / Apache-2.0 | Integração com o runtime Microsoft Edge WebView2 nativo do Windows para o console `PrysmAdmin`. |



## Go (sinalização e API)

| Dependência | Uso no PrysmView | Licença | Motivo de escolha |
|-------------|-------------------|---------|-------------------|
| [`chi`](https://github.com/go-chi/chi) | Roteamento HTTP da API de sinalização | MIT | Leve, idiomático, sem dependências pesadas e adequado para APIs REST simples. |
| [`modernc.org/sqlite`](https://modernc.org/sqlite) | Banco SQLite sem CGO | BSD-3-Clause | Permite build estático multiplataforma sem dependência de libsqlite3 do sistema ou CGO. |
| [`golang.org/x/time/rate`](https://pkg.go.dev/golang.org/x/time/rate) | Rate limiting por IP e por token | BSD-3-Clause | Implementação testada e padrão para limitação de requisições (proteção contra brute-force). |
| [`crypto/ed25519`](https://pkg.go.dev/crypto/ed25519) (stdlib) | Validação de assinaturas Ed25519 do Host | BSD-3-Clause | Presente na biblioteca padrão Go, sem dependência externa adicional. |
| [`golang.org/x/crypto`](https://pkg.go.dev/golang.org/x/crypto) | Hashing de senhas com Argon2id para contas locais de emergência (S-11) | BSD-3-Clause | Padrão oficial do ecossistema Go para derivação de chaves e hash resistente a GPUs/ASICs. |

*Observação:* Versões exatas serão fixadas em `Cargo.lock` e `go.sum` na Fase 0, conforme regras do projeto.