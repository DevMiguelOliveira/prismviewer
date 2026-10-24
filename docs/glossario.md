# Glossário Técnico — PrysmView

- **PDV (Ponto de Venda):** Terminal de caixa de loja rodando Windows 10 ou 11 onde o software do agente (`prysmview-svc` e `prysmview-tray`) é executado.
- **Host (`prysmview-svc`):** Serviço Windows de segundo plano em execução na conta `LocalSystem`. Responsável pela captura de tela GPU, injeção de comandos de entrada, gestão de conexões QUIC e criptografia.
- **Tray (`prysmview-tray`):** Aplicação de bandeja do Windows em execução na sessão interativa do usuário logado no caixa. Responsável por exibir a janela de consentimento e botão de encerramento de sessão.
- **Viewer (`prysmview-viewer`):** Aplicativo desktop nativo do operador construído em Rust com interface gráfica `eframe`/`egui`.
- **Relay (`prysmview-relay`):** Servidor `iroh-relay` que atua como fallback quando a conexão direta P2P entre Viewer e Host é bloqueada por NAT restritivo ou firewall UDP.
- **Sinalização (`prysmview-api`):** Serviço REST/HTTPS em Go que gerencia o registro de PDVs, autenticação de operadores e coordenação de sessões.
- **iroh:** Biblioteca em Rust que implementa transporte QUIC com TLS 1.3, descoberta de pares P2P, NAT traversal (STUN/UPnP) e fallback contínuo para relays.
- **DXGI Desktop Duplication (`IDXGIOutputDuplication`):** API Win32 de aceleração de vídeo em nível de GPU (DirectX 11) que captura a tela do monitor diretamente da memória de vídeo.
- **SendInput:** Função da API do Windows para sintetizar eventos de teclado e mouse.
- **Scan Code:** Código numérico de hardware da tecla no teclado físico, utilizado no PrysmView no lugar de códigos virtuais para preservar a compatibilidade nativa com o layout ABNT2.
- **DPAPI (Data Protection API):** Mecanismo de segurança do Windows (`CryptProtectData`) que cifra segredos no disco atrelados à máquina ou ao usuário.
- **Postcard:** Formato binário de serialização extremamente compacto e `#[no_std]` compatível, utilizado nos frames de protocolo.
- **zstd:** Algoritmo de compressão de alta velocidade utilizado nos tiles de tela (nível 1) e arquivos (nível 3).
- **Zeroize:** Prática e crate de segurança que sobrescreve com zeros a memória RAM contendo chaves privadas e senhas ao serem descartadas.
