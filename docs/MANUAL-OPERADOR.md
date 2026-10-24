# Manual de Operação e Guia de Uso — PrysmView

Este manual orienta o operador de suporte e o usuário do PDV (caixa de loja) sobre o ciclo de vida completo de uma sessão de acesso remoto no PrysmView.

---

## 1. Guia do Operador (PrysmView Viewer)

O aplicativo do operador (`prysmview-viewer`) é construído em Rust (`eframe`/`egui`) com interface escura e funcional inspirada no TeamViewer.

### 1.1 Conexão e Autenticação
1. Abra o `prysmview-viewer.exe`.
2. Insira suas credenciais de operador (usuário e senha) para autenticar na API de sinalização.
3. No catálogo de lojas e caixas, localize o PDV desejado utilizando a barra de busca por nome da loja ou identificador do caixa.
4. Clique no botão **Conectar**.

### 1.2 Código de Sessão de 8 Dígitos
1. Uma solicitação de conexão é enviada ao PDV.
2. Quando o usuário do PDV aceitar, um **código numérico de 8 dígitos** será gerado e exibido na tela do PDV.
3. Solicite o código ao usuário da loja e digite-o no modal do Viewer.
   - **Validade do código**: 120 segundos.
   - **Tentativas máximas**: 3 tentativas (após 3 erros, a sessão é bloqueada por segurança).

### 1.3 Recursos Durante a Sessão
- **Controle de Mouse e Teclado**: Coordenadas de mouse normalizadas (`0..65535`) e teclado por scan codes físicos (layout ABNT2 respeitado).
- **HUD de Telemetria**: Indicador em tempo real no canto superior direito exibindo FPS atual e latência de rede (RTT em ms).
- **Transferência de Arquivos**: Envio e recebimento em blocos de 1 MiB com validação SHA-256.
- **Área de Transferência (Clipboard)**: Sincronização de texto de até 1 MB (respeitando a política local do PDV).
- **Chat de Texto**: Janela de mensagens rápidas (até 4 KiB por mensagem).
- **Ctrl+Alt+Del (SAS)**: Botão no menu superior para acionar a sequência de atenção no desktop Winlogon.
- **Reinício do PDV**: Reinicialização com gravação de estado e reconexão automática.

### 1.4 Encerramento
Clique em **Desconectar** a qualquer momento. Todas as teclas e modificadores serão liberados imediatamente.

---

## 2. Guia do Usuário do PDV (Caixa de Loja / PrysmView Tray)

O aplicativo da bandeja (`prysmview-tray.exe`) executa na sessão de usuário do operador de caixa e garante total visibilidade e consentimento sobre o acesso.

### 2.1 Janela de Consentimento
Quando um operador de suporte solicitar conexão:
1. Uma janela de diálogo é exibida no centro da tela informando:
   - Nome e ID do operador;
   - Motivo da solicitação;
   - Cronômetro regressivo de **60 segundos**.
2. **Aceitar**: Clique em "Aceitar" para permitir o acesso. O código de 8 dígitos será exibido.
3. **Recusar**: Clique em "Recusar" ou aguarde os 60 segundos expirarem para recusar automaticamente.

### 2.2 Indicador de Sessão Ativa e Encerramento Imediato
Durante todo o período em que houver um operador conectado:
1. Um banner indicador de sessão ativa permanece visível na tela.
2. O botão **"Encerrar Sessão Imediatamente"** permite ao usuário da loja interromper o acesso a qualquer instante com efeito instantâneo.
3. Nenhuma sessão oculta ou silenciosa é permitida pelo protocolo.
