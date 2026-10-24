# PrysmView — Documento Geral do Projeto

## 0. Missão
PrysmView é um software de acesso remoto desktop para PDVs (caixas de loja) rodando Windows 10 e Windows 11.
Permite que um operador visualize e controle o PDV remotamente.
Requisitos fundamentais:
- **Rápido:** Latência percebida baixa, fluidez de vídeo aceitável em rede de loja modesta.
- **Leve:** Uso baixo de CPU (< 5% ocioso, < 15% em movimento), memória (< 150 MB) e disco no PDV.
- **Seguro:** Criptografia fim a fim, consentimento obrigatório e visível no PDV, auditoria completa de sessões.
- **Custo Zero:** Nenhuma ferramenta, biblioteca, serviço, domínio ou certificado pago. Custo total: R$ 0,00.

Interface visual inspirada no TeamViewer, em estilo limpo e funcional.

---

## 1. Atores
- **Operador:** Inicia a sessão a partir de outro computador (Viewer).
- **Usuário da Loja:** Pessoa física no caixa do PDV. Vê todo acesso e pode recusá-lo ou encerrá-lo a qualquer momento.
- **Administrador:** Cadastra PDVs, operadores e revoga acessos via API e banco SQLite.

---

## 2. Fluxo Exato de uma Sessão (12 Passos)
1. O host (`prysmview-svc`) inicia como serviço Windows (LocalSystem) e gera, na primeira execução, um par de chaves Ed25519. A pública é a identidade do PDV; a privada é protegida por DPAPI (escopo de máquina).
2. O host se registra na sinalização com: chave pública, nome da loja, número do PDV, versão do agente e status.
3. O operador faz login na API de sinalização (token opaco de vida curta com hash SHA-256 no banco).
4. O operador seleciona o PDV. A API valida as permissões de acesso.
5. A API informa ao host, por canal de sinalização, a solicitação de sessão (id do operador e horário).
6. Se o operador não estiver na lista local de operadores confiáveis, o host aciona a bandeja (`prysmview-tray`) e exibe a janela de consentimento no PDV com nome do operador, motivo informado e botões Aceitar/Recusar (timeout: 60s).
7. Ao aceitar, o host gera um código de sessão de 8 dígitos (validade: 120s, máx 3 tentativas) exibido na tela do PDV. O operador digita esse código no Viewer. A verificação é via HMAC com comparação em tempo constante.
8. O Viewer conecta-se ao host via `iroh` (QUIC com TLS 1.3), usando a chave pública do PDV como endereço.
9. Dentro da conexão, o stream 0 (controle) executa handshake de versão, validação do código e negociação de permissões.
10. Streams dedicados são abertos: vídeo (unidirecional host->viewer), entrada (unidirecional viewer->host), chat e arquivos.
11. Durante a sessão, o indicador visual no PDV permanece visível e com botão de encerramento imediato.
12. Ao encerrar (por qualquer parte, queda de rede ou limite de 4 horas), o host solta todas as teclas e modificadores, descarta chaves da memória (`zeroize`) e grava registro de auditoria.

---

## 3. Escopo da Versão 1

### Incluído:
- Visualização de 1 monitor por vez (com seleção de monitor).
- Controle de mouse e teclado (scan codes ABNT2).
- Transferência de arquivos com validação de hash SHA-256.
- Área de transferência compartilhada (texto, máx. 1 MB, desativável no PDV).
- Chat de texto simples.
- Reinício do PDV com reconexão automática.
- Ctrl+Alt+Del a partir do Viewer via serviço SYSTEM.
- Atualização automática assinada via Ed25519.
- Auditoria de sessões.

### Fora de Escopo na V1:
- Gravação de vídeo de sessão.
- Acesso sem consentimento prévio.
- Acesso via navegador Web.
- Transmissão de áudio.
- Múltiplas sessões simultâneas no mesmo PDV.
- Múltiplos monitores simultâneos.
- Host em macOS ou Linux.

---

## 4. Metas Não-Funcionais Mensuráveis
- Latência de entrada (clique até reflexo): < 150 ms (RTT de 20 ms).
- Captura de vídeo em 1080p tela parada: CPU < 5%.
- Captura de vídeo em movimento moderado: CPU < 15%.
- Consumo de memória do serviço: < 150 MB.
- Consumo de banda: < 50 kbps (tela parada); < 1 Mbps (navegação de texto).
- Suporte a DPI: 100%, 125%, 150% e 175%.
- Tamanho do instalador: < 30 MB.

---

## 5. Operação, Backup e Recuperação de Desastre (Disaster Recovery)

### 5.1 Infraestrutura em Nuvem (Custo Zero)
- **Hospedagem**: Oracle Cloud Always Free (ARM Ampere A1, 4 OCPUs, 24 GB RAM, Ubuntu Server 24.04 LTS).
- **Serviços**: Gerenciados via systemd (`prysmview-api.service` e `prysmview-relay.service`).
- **Firewall**: UFW liberando apenas portas 22 (SSH restrito), 443 TCP (HTTPS API) e 3340 UDP (Relay iroh).

### 5.2 Backup Diário e Retenção
- **Banco de Dados**: SQLite puro sem CGO (`/var/lib/prysmview/prysmview.db`).
- **Rotina de Backup**: Script diário `deploy/backup-sqlite.sh` executado via cron/systemd timer gerando dump comprimido com gzip (`.sql.gz`).
- **Retenção**: Rotação automática de 30 dias.
- **Sincronização**: Envio diário encriptado para repositório Git privado.

### 5.3 Procedimento de Disaster Recovery (RTO < 30 min, RPO < 24 h)
1. Instanciar nova VM Ubuntu Server 24.04 na Oracle Cloud.
2. Executar script de firewall: `bash deploy/setup-ufw.sh`.
3. Instalar binários compilados `prysmview-api` e `prysmview-relay` em `/usr/local/bin/`.
4. Restaurar o último dump SQLite descompactando em `/var/lib/prysmview/prysmview.db` com permissão `0600`.
5. Ativar serviços: `systemctl enable --now prysmview-api prysmview-relay`.