# PrysmView

Software de acesso remoto desktop para PDVs de lojas (Windows 10 e 11).
Permite que um operador visualize e controle um PDV remotamente, com
consentimento visível no PDV.

## Objetivos
- Veloz e leve em hardware modesto
- Seguro em todas as pontas (criptografia, senha de sessão, auditoria)
- Custo zero: nenhuma ferramenta, biblioteca ou serviço pago

## Decisões já tomadas
- Transporte: QUIC com TLS 1.3 via iroh/quinn (ADR 0001)
- Host e viewer: Rust, com windows, eframe/egui
- Sinalização e API: Go, com SQLite
- Relay: iroh em VM Oracle Cloud Always Free
- Captura: DXGI Desktop Duplication
- Injeção de entrada: SendInput

## Restrições
- Teclado ABNT2 é o padrão nas lojas
- Tela de logon e Ctrl+Alt+Del exigem serviço SYSTEM
- Windows 10 precisa de atualização de segurança paga ou migração