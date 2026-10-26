# Changelog — PrysmView

Todas as alterações notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

---

## [0.1.0] - 2026-10-25

### Adicionado
- **Segurança Crítica (Fase 0 e Fase 1):**
  - Assinatura digital Ed25519 em manifestos de atualização e guarda local anti-rollback com reversão atômica (`UpdateBackupGuard`).
  - Emissão de Tickets Criptográficos de Sessão assinados pelo servidor central, com validação offline e cache anti-replay.
  - Identidade única e imutável de PDV por chave pública Ed25519 e proteção de chave privada via DPAPI de máquina.
  - Autenticação corporativa de técnicos via Microsoft Entra ID (OIDC com PKCE) e revogação central síncrona.
  - Proteção estrita contra força bruta e DoS: limite de taxa com bloqueio progressivo por IP e validação de limites de tamanho antes da alocação de heap.
  - Comparação em tempo constante com `subtle::ConstantTimeEq` para códigos e segredos criptográficos.
  - Cadeia de auditoria inviolável append-only com hash encadeado SHA-256.
- **Operação e Resiliência (Fase 2):**
  - Rotina de backup criptografado do banco SQLite com `age` (`deploy/backup-sqlite.sh`) e restauração com validação de integridade.
  - Runbook de Recuperação de Desastres (`docs/runbook-disaster-recovery.md`) com RTO (< 10 min) e RPO (< 1h) medidos.
  - Script de deploy e atualização sem compilação em produção (`deploy/azure-setup.sh`) com rollback atômico em < 2s (`deploy/rollback.sh`).
  - Monitoramento ativo de heartbeats com alerta de queda abrupta (`deploy/check-heartbeat-drop.sh`).
- **Desempenho e Transporte (Fase 3):**
  - Otimização do pipeline de captura DXGI Desktop Duplication utilizando dirty rects para atualização seletiva de tiles 64x64.
  - Keyframes adaptativos suprimindo rajadas de rede em telas estáticas de PDV.
  - Repintura GPU reativa no viewer (`RemoteCanvas`) eliminando uploads redundantes de texturas.
  - Arquitetura de fallback de transporte em 3 camadas (P2P Direto -> Relay UDP 3340 -> Fallback HTTPS/TCP 443).
  - Suporte total a DPI escalonado (100%, 125%, 150%, 175%) com mapeamento de scan codes ABNT2.
- **Interface e Usabilidade (Fase 4):**
  - Sistema de tokens de design centralizado em `theme.rs` com conformidade de contraste WCAG 2.1 AA (> 4.5:1).
  - Centralização de strings em português do Brasil simples e orientadas ao operador de caixa (`strings.rs`).
  - Compilação dos binários finais com subsistema nativo do Windows (`#![windows_subsystem = "windows"]`), sem prompts residuais.
  - Remoção de segredos e IPs públicos das interfaces e KPIs de conectividade.
- **Implantação e Gestão de Frota (Fase 5):**
  - Scripts PowerShell para implantação automatizada em anéis (`Deploy-PrysmHost.ps1`), desinstalação limpa e fila de retentativa offline (`Sync-DeviceCredential.ps1`).
  - Geração de credenciais de máquina com CSPRNG e proteção DPAPI.
  - Capacidade de filtragem e busca ágil em frotas de mais de 1.000 máquinas distribuídas por lojas e franquias.
  - Exportação de relatórios de auditoria em CSV com cadeia criptográfica SHA-256 validada.
- **Governança e Qualidade (Fase 6):**
  - Eliminação de contradições documentais sobre hospedagem, portas e arquitetura.
  - Separação definitiva em dois instaladores especializados Inno Setup 6: `PrysmHost-Setup` (PDV) e `PrysmAdmin-Setup` (Técnico).
  - Formalização de 6 ADRs arquiteturais obrigatórios (ADR 0004 a ADR 0009).
  - Suíte completa de mais de 170 testes unitários e de integração verdes em Rust e Go.
