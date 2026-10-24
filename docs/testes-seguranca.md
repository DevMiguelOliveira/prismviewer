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
