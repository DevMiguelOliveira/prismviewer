# Guia de Deploy na Nuvem Azure - PrysmView

Este guia detalha os passos para realizar o deploy completo dos componentes de nuvem do PrysmView (`prysmview-api` e `prysmview-relay`) em uma máquina virtual Microsoft Azure.

---

## 1. Especificações da Máquina Virtual

- **Família / Tamanho:** `Standard D2alds_v7`
  - **Processador:** Ampere® Altra® ARM64 (2 vCPUs)
  - **Memória RAM:** 8 GiB
  - **Armazenamento:** Disco Premium SSD com disco temporário
- **Sistema Operacional:** Ubuntu Server 24.04 LTS - Gen2
- **Tipo de Autenticação:** Senha (SSH habilitado com usuário/senha)

---

## 2. Configuração de Rede no Azure (Network Security Group - NSG)

No Portal da Azure, acesse sua Máquina Virtual > **Networking (Rede)** > **Inbound port rules (Regras de porta de entrada)** e adicione as seguintes regras:

| Prioridade | Nome da Regra | Porta | Protocolo | Origem | Ação | Descrição |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **1000** | `Allow-SSH` | `22` | TCP | IP do administrador | Permitir | Acesso administrativo remoto — **restringir a IP fixo do admin antes de produzir** |
| **1010** | `Allow-HTTP` | `80` | TCP | Qualquer | Permitir | Descoberta / Let's Encrypt |
| **1020** | `Allow-HTTPS` | `443` | TCP | Qualquer | Permitir | API de Sinalização do PrysmView |
| **1030** | `Allow-Relay-QUIC` | `3340` | UDP | Qualquer | Permitir | Tráfego P2P Relay (iroh/QUIC) |

---

## 3. Conexão SSH via Senha

Abra seu terminal (PowerShell, Windows Terminal ou Bash) no seu computador e conecte-se com as credenciais criadas no portal:

```bash
ssh <usuario_azure>@<IP_PUBLICO_DA_VM>
```

*Exemplo:*
```bash
ssh azureuser@20.195.45.120
```
Digite a senha quando solicitada.

---

## 4. Deploy Automatizado (One-Command)

Assim que estiver conectado na VM, execute os comandos abaixo para baixar o repositório e rodar o script de instalação automática:

```bash
# 1. Clone o repositório (ou envie via SCP/Git)
git clone https://github.com/<seu-usuario>/PrysmView.git
cd PrysmView

# 2. Execute o instalador automatizado com privilégios de root
sudo bash deploy/azure-setup.sh
```

### O que o script faz automaticamente:
1. Atualiza o sistema e instala pacotes essenciais (`build-essential`, `sqlite3`, `ufw`, `git`, `curl`).
2. Cria o usuário restrito de sistema `prysmview` com diretório em `/var/lib/prysmview`.
3. Instala os toolchains oficiais Go e Rust para ARM64.
4. Compila nativamente em ARM64 a API de sinalização (`prysmview-api`) e o Relay (`prysmview-relay`).
5. Instala e inicia os serviços systemd com reinício automático em caso de falha.
6. Configura o firewall UFW interno para proteção contra brute-force.
7. Agenda o cron diário às 03:00 para backup com rotação de 30 dias do banco SQLite.

---

## 5. Verificação da Instalação

Após a execução, valide se ambos os serviços estão em execução (`active (running)`):

```bash
# Status da API de sinalização
sudo systemctl status prysmview-api

# Status do Relay iroh
sudo systemctl status prysmview-relay
```

### Teste de Health Check:
```bash
curl http://localhost:8080/health
```
Resposta esperada:
```json
{"status":"ok","version":"1.0.0"}
```

---

## 6. Configuração de Domínio e HTTPS Automático (Caddy)

Para expor a API publicamente com certificado TLS (Let's Encrypt) automático e gratuito, instale o Caddy e configure o Caddyfile com redirecionamento HTTP→HTTPS, cabeçalho HSTS e terminação TLS:

```bash
# Instale o Caddy
sudo apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt-get update
sudo apt-get install caddy -y

# Configure o Caddyfile (/etc/caddy/Caddyfile)
sudo tee /etc/caddy/Caddyfile > /dev/null <<'EOF'
# HTTP para HTTPS redirect
:80 {
    redir https://{host}{uri} permanent
}

# HTTPS com terminação TLS (Let's Encrypt automático)
api.seudominio.com {
    # HSTS: navegadores forçam HTTPS por 1 ano, inclui subdomínios, pré-carregamento
    header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"

    # Remove identificador do servidor de cabeçalhos de resposta
    header / Server ""

    # Reverse proxy para a API Go em localhost:8080 (HTTP puro internamente)
    reverse_proxy 127.0.0.1:8080 {
        header_up Host {upstream_hostport}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-For {remote}
        header_up X-Forwarded-Proto {scheme}
        header_up X-Forwarded-Port {server_port}
    }

    # Certificado automático via Let's Encrypt
    tls {
        protocolo tls1.2 tls1.3
    }
}
EOF

# Ajuste o servico systemd da API para ouvir em 127.0.0.1:8080 (Caddy encerra TLS em :443)
sudo sed -i 's/-addr :443/-addr :8080/' /etc/systemd/system/prysmview-api.service
sudo systemctl daemon-reload
sudo systemctl restart prysmview-api

# Reinicie o Caddy
sudo systemctl restart caddy
```

> **Nota:** Para obter nota A no SSL Labs, é necessário um domínio real com certificado Let's Encrypt emitido pelo Caddy. A configuração acima já inclui HSTS, redirect permanente e suítes TLS modernas (TLS 1.2/1.3).

---

## 7. Logs e Diagnóstico em Tempo Real

Para monitorar os logs dos serviços:

```bash
# Logs da API de sinalização
sudo journalctl -u prysmview-api -f

# Logs do Relay iroh
sudo journalctl -u prysmview-relay -f
```
