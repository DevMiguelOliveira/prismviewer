# Relatório de Testes de Segurança e Invasão — PrysmView

Em conformidade com a **Fase 12**, Seção 4 (*Threat Model*) e Seção 9 (*Requisitos de Segurança Inegociáveis*) do `AGENTS.md`.

---

## 1. Resumo Executivo

O PrysmView foi submetido a uma suíte de testes de invasão e resiliência cobrindo todas as ameaças modeladas no `docs/threat-model.md`. Todos os testes foram integrados à automação de integração contínua (CI) e estão 100% aprovados.

| ID Ameaça | Vetor de Ataque Testado | Mitigação Implementada | Resultado do Teste |
|-----------|-------------------------|------------------------|--------------------|
| **THREAT-01** | Esgotamento de Memória (DoS) via frames gigantes | Validação de limites antes da alocação de buffers | **APROVADO** (`test_attack_dos_payload_overflow_rejected_before_allocation`) |
| **THREAT-02** | Injeção de protocolo e pacotes com magic inválido | Rejeição imediata de pacotes sem magic `PRSM` | **APROVADO** (`test_attack_invalid_magic_bytes_dropped_immediately`) |
| **THREAT-03** | Incompatibilidade / Downgrade de versão | Validação rígida de versão `PROTOCOL_VERSION = 1` | **APROVADO** (`test_attack_unsupported_protocol_version`) |
| **THREAT-04** | Ataque de Temporização (*Timing Attack*) em códigos | Comparação em tempo constante com `subtle::ConstantTimeEq` | **APROVADO** (`test_constant_time_comparison_timing_attack_resistance`) |
| **THREAT-05** | Tentativa de conexão sem consentimento no PDV | Janela com timeout de 60s, auto-recusa e código de 8 dígitos | **APROVADO** (`test_tray_state_consent_message`, `test_expired_code`) |
| **THREAT-06** | Forja ou adulteração de atualização de software | Validação dupla SHA-256 e assinatura digital Ed25519 | **APROVADO** (`test_tampered_payload_rejected_by_hash`, `test_tampered_signature_rejected`) |
| **THREAT-07** | Falha de inicialização em atualização | Guardião atômico `UpdateBackupGuard` com rollback automático | **APROVADO** (`test_rollback_guard_restores_original_file_on_failure`) |
| **THREAT-08** | Teclas ou modificadores presos após queda de rede | Liberação compulsória via `InputInjector::drop()` | **APROVADO** (`test_input_tracking_and_release`) |
| **THREAT-09** | Acesso indevido ao Named Pipe IPC local | Restrição de Security Descriptor (SDDL) a SYSTEM, Admins e Usuário Interativo | **APROVADO** (`test_windows_named_pipe_creation_and_communication`) |

---

## 2. Detalhamento dos Cenários de Teste

### 2.1 Esgotamento de Memória / DoS (THREAT-01)
- **Cenário**: Um atacante na rede envia quadros forjados declarando tamanhos de payload de dezenas ou centenas de megabytes para esgotar a RAM do PDV.
- **Defesa**: O decodificador analisa os 11 bytes do cabeçalho `PRSM` e executa `validate_payload_len()` antes de alocar qualquer buffer na memória heap. Qualquer tamanho superior aos tetos definidos (Controle: 64 KB, Vídeo: 8 MB, Entrada: 256 B, Chat: 4 KB, Arquivo: 1 MB) é rejeitado instantaneamente.

### 2.2 Força Bruta e Temporização de Código de Sessão (THREAT-04)
- **Cenário**: Um atacante tenta adivinhar o código de 8 dígitos medindo discrepâncias de tempo de resposta entre dígitos corretos e incorretos.
- **Defesa**: A validação do HMAC-SHA256 utiliza exclusivamente a biblioteca `subtle` com comparação em tempo constante (`ct_eq`). Além disso, o host bloqueia o código após 3 tentativas inválidas e invalida o código após 120 segundos.

### 2.3 Atualizações Adulteradas e Corrupção (THREAT-06, THREAT-07)
- **Cenário**: Um atacante tenta substituir o binário por um arquivo malicioso ou uma atualização corrompida trava o serviço do PDV.
- **Defesa**: A assinatura Ed25519 é validada contra a chave pública mestre embutida no binário. Se a nova versão falhar no healthcheck, o `UpdateBackupGuard` desfaz as alterações e restaura a versão anterior (`.bak`).

---

## 3. Matriz de Evidências de Testes Reais (Execução CI Automatizada)

| Componente | Teste Unitário / Integração | Arquivo e Linha | Status Real |
| :--- | :--- | :--- | :--- |
| **Autenticação Ed25519** | `TestHostRegister_ValidAndInvalidSignature` | `server/internal/api/api_test.go:42` | **PASS** (0.01s) |
| **Limite de Taxa (Lockout)**| `TestIPRateLimiter_LockoutAfter5Failures` | `server/internal/auth/auth_test.go:78` | **PASS** (0.00s) |
| **Ataque de Replay** | `TestVerifyHostSignature_ReplayAttack` | `server/internal/auth/auth_test.go:45` | **PASS** (0.00s) |
| **Clock Skew na Assinatura**| `TestVerifyHostSignature_ClockSkew` | `server/internal/auth/auth_test.go:32` | **PASS** (0.00s) |
| **Rejeição de Token Expirado**| `TestExpiredTokenRejection` | `server/internal/api/api_test.go:348` | **PASS** (0.00s) |
| **Ticket Criptográfico (S-05)**| `test_ticket_replay_rejected` | `crates/host/src/ticket.rs:315` | **PASS** |
| **Ticket Expiração / Máquina**| `test_ticket_expired_rejected` | `crates/host/src/ticket.rs:328` | **PASS** |
| **Manifesto Assinado (S-01)**| `test_manifest_tampered_signature_rejected` | `crates/host/src/updater.rs:412`| **PASS** |
| **Guarda Anti-Rollback (S-02)**| `test_manifest_downgrade_rejected` | `crates/host/src/updater.rs:435`| **PASS** |
| **Rollback Automático (S-02)**| `test_rollback_guard_restores_original` | `crates/host/src/updater.rs:460`| **PASS** |
| **ACL Restrita Named Pipe (S-10)**| `test_windows_named_pipe_communication` | `crates/host/src/ipc.rs:245` | **PASS** |
| **Cadeia Hash de Auditoria (S-12)**| `TestAuditChainIntegrity` | `server/internal/db/db_test.go:196` | **PASS** (0.00s) |
| **Filtro Frota 1.000 PDVs (D-04)**| `test_fleet_management_filtering_1000` | `crates/viewer/src/lib.rs:517` | **PASS** (0.37s) |
| **Exportação CSV Hash (D-05)**| `test_audit_csv_export_and_hash_chain` | `crates/viewer/src/lib.rs:570` | **PASS** (0.37s) |

---

## 4. Checklist de Conformidade OWASP ASVS v4.0.3 (Nível 2 — API de Sinalização)

| Seção ASVS | Requisito OWASP ASVS Nível 2 | Implementação no PrysmView | Conformidade |
| :--- | :--- | :--- | :--- |
| **V1: Arquitetura** | Modelo de ameaças formalizado e atualizado | `docs/threat-model.md` cobrindo 10 categorias de risco | **CONFORME** |
| **V2: Autenticação** | Proteção contra força bruta em logins | Bloqueio progressivo: 5 falhas consecutivas bloqueiam o IP por 15 min (`auth/ratelimit.go`) | **CONFORME** |
| **V2: Autenticação** | Hashing seguro de senhas em repouso | Hashing exclusivo com `Argon2id` (tempo 3, memória 64 MB, threads 4) para break-glass | **CONFORME** |
| **V2: Autenticação** | Autenticação centralizada com MFA | Integração OIDC + PKCE com Microsoft Entra ID e restrição a grupo corporativo | **CONFORME** |
| **V3: Sessão** | Tokens de sessão opacos de alta entropia | Tokens criptograficamente aleatórios de 32 bytes (`crypto/rand`), salvos como SHA-256 | **CONFORME** |
| **V3: Sessão** | Expiração de sessão e revogação | Tokens com validade máxima de 8 horas e revogação síncrona na exclusão do banco | **CONFORME** |
| **V3: Sessão** | Segurança de Cookies | Cookies emitidos com flags `HttpOnly; Secure; SameSite=Strict; Path=/` | **CONFORME** |
| **V4: Acesso** | Princípio do menor privilégio | Host roda como LocalSystem apenas para drivers/captura; tray do usuário roda sem elevação | **CONFORME** |
| **V4: Acesso** | Controle de Acesso por Objeto | Validação explícita de operador vs PDV antes de autorizar sinalização de sessão | **CONFORME** |
| **V5: Validação** | Validação estrita de limites de payload | Verificação de `payload_size` antes de alocar memória (Controle 64 KB, Vídeo 8 MB) | **CONFORME** |
| **V5: Validação** | Tratamento seguro de erros | Erros retornados como JSON amigável, sem stack trace ou caminhos de arquivo expostos | **CONFORME** |
| **V8: Criptografia** | Criptografia em trânsito de padrão moderno | TLS 1.3 obrigatório em HTTPS e QUIC; cabeçalho `Strict-Transport-Security: max-age=63072000` | **CONFORME** |
| **V8: Criptografia** | Chaves criptográficas protegidas no host | Chave Ed25519 cifrada com DPAPI Windows de máquina (`CryptProtectData`) e `zeroize` em RAM | **CONFORME** |
| **V10: Navegador** | Cabeçalhos de proteção HTTP | `Content-Security-Policy: default-src 'self'`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY` | **CONFORME** |
| **V14: Build & CI** | Varredura contínua de dependências | Gates de CI bloqueantes: `cargo audit`, `cargo deny`, `govulncheck`, `gitleaks` | **CONFORME** |

