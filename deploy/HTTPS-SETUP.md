# Configuração HTTPS - PrysmView

Este documento descreve a configuração HTTPS usando Caddy com certificados Let's Encrypt automáticos.

## Arquitetura

```
Internet (HTTPS)
       ↓
Caddy (443) - Terminação TLS + Let's Encrypt
       ↓
API Go (8080) - HTTP interno (localhost)
```

## Configuração

### 1. Caddyfile

O arquivo `deploy/Caddyfile` configura:
- Redirecionamento automático HTTP → HTTPS
- Terminação TLS com certificados Let's Encrypt
- Proxy reverso para a API Go em `localhost:8080`
- HSTS para forçar HTTPS
- TLS 1.2 e 1.3 apenas

### 2. Serviço Systemd

A API Go (`prysmview-api`) é configurada para:
- Escutar em `127.0.0.1:8080` (não exposta publicamente)
- O Caddy lida com HTTPS e encaminha para a API

### 3. Firewall UFW

Portas abertas:
- **80 (TCP)**: HTTP para redirecionamento e Let's Encrypt
- **443 (TCP)**: HTTPS para a API
- **22 (TCP)**: SSH (restringir a IP do admin em produção)
- **3340 (UDP)**: Relay iroh (WebRTC)

## Deploy

### Opção 1: Script Automatizado (azure-setup.sh)

```bash
sudo bash deploy/azure-setup.sh
```

Este script:
1. Instala o Caddy
2. Configura o Caddyfile
3. Inicia o serviço Caddy
4. Configura o firewall UFW

### Opção 2: Deploy Manual (deploy-to-azure.sh)

```bash
./deploy/deploy-to-azure.sh azureuser@vm-ip
```

### Opção 3: Manual em VM Azure

```bash
# 1. Instalar Caddy
sudo apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt-get update
sudo apt-get install caddy -y

# 2. Copiar Caddyfile
sudo cp deploy/Caddyfile /etc/caddy/Caddyfile

# 3. Reiniciar Caddy
sudo systemctl restart caddy
sudo systemctl enable caddy
```

## Domínio

O domínio configurado é: **prysmview.duckdns.org**

### Configuração do DuckDNS

1. Acesse [duckdns.org](https://www.duckdns.org)
2. Configure o subdomínio `prysmview` para apontar para o IP público da VM Azure
3. Aguarde a propagação DNS (geralmente instantânea)

### Configuração do NSG Azure

No Azure Network Security Group, adicione:
- **Porta 80 (TCP)**: Permitir (HTTP)
- **Porta 443 (TCP)**: Permitir (HTTPS)

## Verificação

### 1. Verificar Status do Caddy

```bash
sudo systemctl status caddy
```

### 2. Verificar Logs do Caddy

```bash
sudo journalctl -u caddy -f
```

### 3. Testar Health Check

```bash
curl https://prysmview.duckdns.org/health
```

Resposta esperada:
```json
{"status":"ok","version":"1.0.0"}
```

### 4. Verificar Certificado SSL

```bash
curl -vI https://prysmview.duckdns.org
```

Procure por:
- `TLS 1.2` ou `TLS 1.3`
- `subject: CN=prysmview.duckdns.org`
- `issuer: C=US; O=Let's Encrypt`

### 5. Testar Redirecionamento HTTP → HTTPS

```bash
curl -I http://prysmview.duckdns.org
```

Deve retornar:
```
HTTP/1.1 301 Moved Permanently
Location: https://prysmview.duckdns.org/
```

## Renovação de Certificado

O Caddy renova automaticamente os certificados Let's Encrypt 30 dias antes da expiração. Nenhuma ação manual é necessária.

Para verificar a data de expiração:

```bash
sudo caddy validate-config --config /etc/caddy/Caddyfile
```

## Troubleshooting

### Erro: "Failed to obtain certificate"

- Verifique se o domínio `prysmview.duckdns.org` aponta para o IP correto
- Verifique se a porta 80 está aberta no firewall
- Verifique se o NSG Azure permite porta 80

### Erro: "Connection refused"

- Verifique se o Caddy está rodando: `sudo systemctl status caddy`
- Verifique se a API Go está rodando: `sudo systemctl status prysmview-api`
- Verifique se a API está escutando em 8080: `sudo netstat -tlnp | grep 8080`

### Erro: "502 Bad Gateway"

- Verifique se a API Go está rodando
- Verifique logs da API: `sudo journalctl -u prysmview-api -f`
- Verifique se o Caddy pode conectar em 127.0.0.1:8080

## Segurança

- **HSTS**: Força HTTPS por 1 ano
- **TLS 1.2/1.3**: Protocolos modernos apenas
- **Cabeçalho Server**: Removido para não expor informações
- **HTTP → HTTPS**: Redirecionamento permanente
- **Let's Encrypt**: Certificados automáticos e gratuitos

## Clientes

Os clientes (Admin e Host) já estão configurados para usar:
- `https://prysmview.duckdns.org`

Verifique:
- `apps/admin/src/store/adminStore.ts` (linha 173)
- `apps/host/src/store/hostStore.ts` (linha 43)
