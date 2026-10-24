# Protocolo PRSM v1

## 1. Visão Geral

O protocolo PRSM define o formato de mensagens e a estrutura de comunicação para o software PrysmView, um software de acesso remoto desktop para PDVs (caixas de loja). Todo o comunicação entre o host (agente no PDV) e o viewer (operador) ocorre através de conexões QUIC com TLS 1.3, usando o protocolo definido abaixo.

O protocolo é projetado para ser leve, seguro e eficiente em redes hostis (Wi-Fi de loja, redes mal configuradas). Todas as mensagens são serializadas usando o formato binário compacto `postcard`, permitindo despachamento rápido e análise de protocolo determinística.

---

## 2. Cabeçalho de Quadro

Todo quadro trocado entre pares começa com um cabeçalho fixo de **11 bytes**. O cabeçalho é sempre transmitido em **big-endian** (byte mais significativo primeiro).

### Estrutura do Cabeçalho (11 bytes):

| Offset | Tamanho | Campo | Descrição |
|--------|---------|-------|-----------|
| 0 | 4 bytes | `magic` | Assinatura mágica ASCII "PRSM" (0x50, 0x52, 0x53, 0x4D). Usado para verificação imediata do protocolo. |
| 4 | 2 bytes | `protocol_version` | Versão do protocolo. Valor atual: **1** (u16 big-endian). |
| 6 | 1 byte | `message_type` | Tipo da mensagem. Valores definidos: |
| | | | - 0 = Controle (handshake, código de sessão, permissões, encerramento) |
| | | | - 1 = Vídeo (frame de tela comprimido) |
| | | | - 2 = Entrada (mouse/teclado, scan codes ABNT2) |
| | | | - 3 = Chat (texto simples, clipboard) |
| | | | - 4 = Arquivo (bloco de transferência) |
| 7 | 4 bytes | `payload_size` | Tamanho do payload em bytes. Validado contra limites rigorosos antes de qualquer alocação de memória. |

### Limites de Payload por Tipo de Mensagem:

| `message_type` | Tipo | Limite Máximo de `payload_size` |
|----------------|------|---------------------------------|
| 0 | Controle | **64 KiB** (65536 bytes) |
| 1 | Vídeo | **8 MiB** (8388608 bytes) |
| 2 | Entrada | **256 bytes** |
| 3 | Chat | **4 KiB** (4096 bytes) |
| 4 | Arquivo | **1 MiB** (1048576 bytes) |

**Validade:** O `payload_size` é validado rigorosamente **antes** de qualquer alocação de memória. Se o tamanho exceder o limite para o tipo de mensagem, a conexão é imediatamente encerrada e o evento é registrado em auditoria. Isso previne ataques DoS (Denial of Service) por payloads oversized.

---

## 3. Streams QUIC

As conexões QUIC estabelecem entre host e viewer utilizam streams dedicados (unidirecionais ou bidirecionais) para tipos de conteúdo específicos. O fluxo de streams é fixo e negociado durante o handshake inicial.

### Stream 0 — Controle (bidirecional)

- **Finalidade:** Handshake de versão, validação de código de sessão, definição de permissões, encerramento de sessão.
- **Mensagens:** `message_type` = 0 (Controle).
- **Limite:** Payload máximo de 64 KiB por quadro.
- **Fluxo:**
  1. Host envia handshake com versão do protocolo e ID do PDV (chave pública Ed25519).
  2. Viewer envia código de sessão de 8 dígitos (validado por HMAC-SHA256 em tempo constante).
  3. Definição de permissões (vídeo, entrada, arquivos, chat).
  4. Durante a sessão: mensagens de encerramento, troca de permissões.
  5. Ao encerrar: host solta todas as teclas e modificadores, descarta chaves de sessão.

### Stream 1 — Vídeo (unidirecional: host → viewer)

- **Finalidade:** Transmissão de frames de tela do PDV para o viewer.
- **Mensagens:** `message_type` = 1 (Vídeo).
- **Limite:** Payload máximo de 8 MiB por quadro (frame individual).
- **Formato do Payload Vídeo:**
  - `frame_id`: u64 — identificador monotonicamente crescente do frame.
  - `changed_tiles`: u16 — número de tiles de 64x64 pixels que mudaram neste frame.
  - `tile_indices`: Vec<u16> — índices dos tiles que mudaram (máximo 65535 tiles; tela 1920×1080 = 30×17 = 510 tiles).
  - `tile_data`: Vec<u8> — dados de cada tile alterado, comprimidos com **zstd nível 1** (prioridade a latência sobre compressão).
- **Keyframe:** Um keyframe completo é enviado a cada 5 segundos ou sob solicitação explícita do viewer.
- **Taxa de frames:** Meta de 15 a 30 fps, reduzida automaticamente quando a rede piora.
- **Comportamento de canal lento:** Se o canal estiver lento, frames intermediários são descartados (nunca acumulados). No máximo um frame em trânsito por vez.

### Stream 2 — Entrada (unidirecional: viewer → host)

- **Finalidade:** Transmissão de eventos de mouse e teclado do viewer para o host, controlando o PDV.
- **Mensagens:** `message_type` = 2 (Entrada).
- **Limite:** Payload máximo de **256 bytes** por quadro.
- **Formato do Payload Entrada:**
  - `input_subtype`: u8 — tipo de evento (0 = MouseMove, 1 = MouseButton, 2 = Wheel, 3 = Key).
  - Para **MouseMove** (subtipo 0):
    - `mouse_x`: u16 — coordenada X normalizada (0 a 65535) sobre o monitor selecionado.
    - `mouse_y`: u16 — coordenada Y normalizada (0 a 65535) sobre o monitor selecionado.
  - Para **MouseButton** (subtipo 1):
    - `mouse_button`: u8 — identificador do botão (1 = esquerdo, 2 = direito, etc.).
    - `mouse_button_state`: u8 — 0 = solto, 1 = pressionado.
  - Para **Wheel** (subtipo 2):
    - `wheel_delta_v`: i16 — delta vertical da roda do mouse.
    - `wheel_delta_h`: i16 — delta horizontal da roda do mouse.
  - Para **Key** (subtipo 3):
    - `key_scan_code`: u16 — **scan code** (código físico da tecla). **Teclas são enviadas por scan code, nunca por código virtual**, para respeitar o layout ABNT2 do PDV.
    - `key_extended`: u8 — flag de tecla estendida (0 = normal, 1 = tecla estendida, ex.: PrintScreen, AltGr, teclas de função).
    - `key_state`: u8 — 0 = solto, 1 = pressionado.
- **Segurança:** O host rejeita qualquer comando de entrada que chegue sem sessão autenticada. Teclas permanecem presas nunca em cenários de queda de conexão — ao encerrar, todas as teclas e modificadores são soltas.

### Stream 3 — Chat e Clipboard (bidirecional)

- **Finalidade:** Troca de texto simples (chat) e área de transferência (texto).
- **Mensagens:** `message_type` = 3 (Chat).
- **Limite:** Payload máximo de **4 KiB** (4096 bytes) por quadro.
- **Formato do Payload Chat:**
  - `text`: Vec<u8> — texto UTF-8.
  - `text_len`: u16 — length actual do texto em bytes (máximo 4096).
- **Desativável:** O usuário local no PDV pode desativar clipboard e transferência de arquivos via configuração do sistema. Quando desativado, o host ignora mensagens de chat/clipboard entrantes e não envia as saíntes.

### Stream 4 — Arquivos (bidirecionais, por transferência)

- **Finalidade:** Transferência de arquivos entre host e viewer.
- **Mensagens:** `message_type` = 4 (Arquivo).
- **Limite:** Payload máximo de **1 MiB** (1048576 bytes) por bloco (block).
- **Formato do Payload Arquivo:**
  - `block_id`: u32 — identificador do bloco (0, 1, 2, ...).
  - `block_hash`: [u8; 32] — hash **SHA-256** deste bloco de dados. Verificado no destino antes de renomear o arquivo para o nome final.
  - `data`: Vec<u8> — dados do bloco (máximo 1 MiB).
- **Procedimento de Transferência:**
  1. Arquivo é dividido em blocos de até 1 MiB.
  2. Cada bloco é enviado como uma mensagem separada (`message_type` = 4).
  3. No destino: o hash SHA-256 de cada bloco é verificado. Em caso de falha, o bloco parcial é removido e o solicitante retransmite.
  4. Após todos os blocos recebidos e verificados: o arquivo é montado e renomeado para o nome final.
  5. Se qualquer hash não conferir: o arquivo completo é descartado e o transferente reinicia do bloco 0.

---

## 4. Handshake de Versão

Antes de qualquer dado ser trocado, ocorre um handshake de versão para garantir compatibilidade entre host e viewer.

### Fluxo do Handshake:

1. **Host → Viewer (Stream 0):** Host envia mensagem de controle com:
   - `message_type` = 0 (Controle)
   - `payload`: contém `protocol_version` (u16 = 1), `host_id` (chave pública Ed25519 do PDV), e `os_version` (versão do Windows, ex.: "Windows 10 22H2").
   - O host assina todo o handshake com sua chave privada Ed25519 (protegida por DPAPI no disco), nunca transmitindo a chave privada.

2. **Viewer → Host (Stream 0):** Viewer responde com:
   - `message_type` = 0 (Controle)
   - `payload`: contém `protocol_version` (deve ser 1), `viewer_id` (identificador do viewer/operador), e valida a assinatura do host usando a chave pública embutida no viewer.
   - O viewer verifica se a diferença entre sua versão e a do host é **no máximo 1 versão maior**. Se a versão do viewer for diferente da do host em mais de uma versão maior, a sessão é recusada com mensagem clara para ambos.

3. **Validação de Código de Sessão:**
   - Host gera um código de sessão de **8 dígitos** (numérico, 00000000 a 99999999), validade de **120 segundos**, **máximo 3 tentativas**.
   - O código é gerado via **HMAC-SHA256** usando uma chave de sessão compartilhada, com comparação em tempo constante (`subtle::ConstantTimeEq`) para mitigar ataques por temporização.
   - O código é exibido na tela do PDV (via bandeja `prysmview-tray`). O operador digita o código no viewer.
   - O viewer envia o código de volta ao host validado por HMAC-SHA256.
   - Se o código estiver correto e dentro do tempo limite, as permissões são definidas e os streams de vídeo/entrada são abertos.
   - Se o código estiver errado: contador de tentativas incrementa. Após 3 falhas, a sessão é recusada e o IP pode ser bloqueado progressivamente na API de sinalização (5 falhas bloqueiam o IP por 15 minutos).

---

## 5. Código de Sessão (8 dígitos)

### Especificação:

- **Formato:** 8 dígitos numéricos (00000000 a 99999999).
- **Validade:** 120 segundos a partir da geração.
- **Limite de Tentativas:** Máximo 3 tentativas por código.
- **Geração:** O host gera o código via `HMAC-SHA256(chave_mestra, timestamp)`, tomando os primeiros 8 dígitos decimais do resultado. A chave mestra é estabelecida durante o handshake de versão usando Diffie-Hellman ou troca de chaves Ed25519.
- **Comparação:** Comparação em tempo constante usando `subtle::ConstantTimeEq`. **Proibido** usar `==` ou `!=` diretos em dados de rede para comparação de código de sessão.

### Segurança:

- O código de sessão **não deve** ser registrado em logs. Logs de auditoria registram apenas o **resultado** do consentimento (aceito/recusado) e a quantidade de tentativas, nunca o código em si.
- O código expira após 120 segundos, mesmo que não tenha sido usado todas as 3 tentativas.
- Após 3 tentativas erradas, a sessão é encerrada. O usuário local no PDV pode recusar a sessão a qualquer momento via janela de consentimento na bandeja.

---

## 6. Criptografia e Segurança

### Requisitos Inegociáveis:

1. **Toda comunicação entre pares é QUIC com TLS 1.3.** Nunca criar criptografia própria.
2. **Nenhuma sessão inicia sem consentimento visível no PDV,** exceto para operadores já marcados como confiáveis naquele PDV pelo próprio usuário local.
3. **Código de sessão:** 8 dígitos, validade 120s, máximo 3 tentativas, HMAC-SHA256 com comparação em tempo constante.
4. **Bloqueio progressivo após tentativas erradas na API:** 5 falhas bloqueiam o IP por 15 minutos.
5. **Chave privada do PDV nunca sai do disco protegido por DPAPI,** nunca aparece em log, nunca vai para o servidor.
6. **Limites de tamanho validados antes de qualquer alocação de memória.**
7. **unsafe somente com comentário // SAFETY: e teste associado.**
8. **Proibido unwrap() e expect() em código de rede, entrada e arquivos.**
9. **Logs de auditoria registram:** operador, PDV, horário de início e fim, motivo informado, resultado do consentimento e quantidade de bytes transferidos. Nunca registram tela, teclas, conteúdo de arquivos ou códigos.
10. **Ao encerrar uma sessão por qualquer motivo:** todas as teclas e modificadores são soltos e as chaves de sessão são zeradas (zeroized).
11. **A janela de consentimento e o indicador de sessão não podem ser ocultados pelo viewer.**
12. **O host recusa qualquer comando de entrada ou vídeo que chegue sem sessão autenticada.**
13. **Atualização só é aplicada com assinatura Ed25519 válida.**
14. **Transferência de arquivos:** verificação SHA-256 no destino antes de renomear o arquivo para o nome final. Em caso de falha, o arquivo parcial é removido.
15. **Clipboard e transferência de arquivos podem ser desativados pelo usuário local no PDV.**

### Superfícies de Ataque Mitigadas:

- **Interceptação (Eavesdropping):** TLS 1.3 em toda comunicação peer-to-peer. O relay iroh vê apenas tráfego criptografado, nunca o conteúdo.
- **Replay Attack:** Nonce + timestamp em requisições do host para sinalização. Código de sessão com validade 120s. Chaves de sessão descartadas ao encerrar.
- **MITM (Man-in-the-Middle):** TLS 1.3 com autenticação mútua via chaves. Endereçamento por chave pública do PDV no iroh. Verificação de assinatura Ed25519 em atualizações.
- **Token Vazado:** Token opaco de 32 bytes, armazenado apenas como hash SHA-256 no banco, validade 8h. Bloqueio progressivo: 5 falhas bloqueiam IP por 15 min. Código de sessão obrigatório mesmo com token válido. Consentimento local obrigatório (exceto operadores confiáveis). Auditoria completa.
- **Acesso sem Consentimento:** Janela de consentimento visível obrigatória. Timeout 60s = recusa. Lista de operadores confiáveis apenas definida pelo usuário local. Indicador de sessão sempre visível com botão de encerramento.
- **Brute Force de Código:** Máximo 3 tentativas. Código com validade 120s. Verificação por HMAC-SHA256 com comparação em tempo constante. 8 dígitos = 100 milhões de combinações.
- **Exposição de Chave Privada:** Protegida por DPAPI (escopo de máquina). Nunca sai do disco. Nunca aparece em log. Nunca vai para servidor.
- **Injeção/Comandos Não Autenticados:** Host recusa qualquer comando de entrada ou vídeo que chegue sem sessão autenticada. Handshake obrigatório no stream de controle.

---

## 7. Tamanhos Validados Antes da Alocação

Todos os tamanhos são validados **antes** de qualquer alocação de memória ou despacho de processamento. A sequência de verificação é:

1. Ler os primeiros 11 bytes do quadro (cabeçalho).
2. Validar `magic` == "PRSM".
3. Validar `protocol_version` == 1 (ou compatível).
4. Validar `message_type` está no intervalo 0-4.
5. Validar `payload_size` contra o limite máximo para o tipo de mensagem (tabela acima).
6. Apenas se todos os acima passarem: alocar memória e processar o payload.

Se qualquer validação falhar, o quadro é descartado e um evento de "quadro inválido" é registrado em auditoria. O protocolo não tenta "adivinhar" ou "recuperar" dados inválidos.

---

## 8. Exemplos de Quadros Codificados

### Exemplo 1: Header de Controle (64 KiB máximo)

```
Offset: 0  1  2  3  4  5  6  7  8  9 10
Magic:  0x50 0x52 0x53 0x4D
Version: 0x00 0x01  (protocol_version = 1)
Type:    0x00  (message_type = Controle)
Size:    0x00 0x00 0x01 0x00  (payload_size = 65536 = 64 KiB)
```

Hex total: `50 52 53 4D 00 01 00 00 01 00 00` (11 bytes)

### Exemplo 2: Header de Vídeo (8 MiB máximo)

```
Offset: 0  1  2  3  4  5  6  7  8  9 10
Magic:  0x50 0x52 0x53 0x4D
Version: 0x00 0x01
Type:    0x01  (message_type = Vídeo)
Size:    0x00 0x07 0xF1 0x80  (payload_size = 8388608 = 8 MiB)
```

Hex total: `50 52 53 4D 00 01 01 00 07 F1 80` (11 bytes)

### Exemplo 3: Header de Entrada (256 bytes máximo)

Hex total para payload de 255 bytes: `50 52 53 4D 00 02 00 00 00 00 FF` (11 bytes, payload_size = 255)

### Exemplo 4: Header de Chat (4 KiB máximo)

Hex total para payload de 4095 bytes: `50 52 53 4D 00 03 00 00 0F FF` (11 bytes, payload_size = 4095)

### Exemplo 5: Header de Arquivo (1 MiB máximo)

Hex total para payload de 1048575 bytes: `50 52 53 4D 00 04 00 00 0F 42 4F` (11 bytes, payload_size = 1048575)

---

## 9. Referências ao Código

### Constantes no Código-fonte:

No `crates/core/src/header.rs`:
- `Header::SIZE = 11` (tamanho fixo do cabeçalho)
- `Header::new(magic, protocol_version, message_type, payload_size)`
- `Header::encode()` e `Header::decode()`
- `Header::validate_payload_size()` — valida limites por tipo

No `crates/core/src/limits.rs`:
- `MAX_CONTROL_PAYLOAD: u32 = 64 * 1024`
- `MAX_VIDEO_PAYLOAD: u32 = 8 * 1024 * 1024`
- `MAX_INPUT_PAYLOAD: u32 = 256`
- `MAX_CHAT_PAYLOAD: u32 = 4 * 1024`
- `MAX_FILE_PAYLOAD: u32 = 1024 * 1024`
- `validate_payload_len(msg_type, len) -> Result<(), LimitError>`

No `crates/core/src/messages.rs`:
- `MESSAGE_CONTROL: u8 = 0`, `MESSAGE_VIDEO: u8 = 1`, etc.
- `ControlPayload`, `VideoPayload`, `InputPayload`, `ChatPayload`, `FilePayload` estruturas
- `encode_message(header, payload) -> Result<Vec<u8>, Error>`
- `decode_message(buf) -> Result<(Header, Vec<u8>>, Error>`

---

## 10. Próximos Passos

A especificação do protocolo PRSM v1 está completa. As implementações seguintes são necessárias:

1. **Implementação do handshake** no `crates/host` e `crates/viewer` usando as estruturas definidas.
2. **Implementação da detecção de tiles** e compressão zstd no fluxo de vídeo (Stream 1).
3. **Implementação da injeção SendInput** com scan codes ABNT2 no fluxo de entrada (Stream 2).
4. **Implementação da transferência de arquivos** com blocos de 1 MiB e verificação SHA-256 no fluxo Stream 4.
5. **Implementação da janela de consentimento** e código de sessão no `prysmview-tray`.
6. **Testes proptest** adicionais cobrindo bordas de limite e cenários de falha.