#!/bin/bash
# deploy-to-azure.sh - Script de deploy do PrysmView para VM Azure
# Uso: ./deploy-to-azure.sh <azure-user@azure-vm>

set -e

if [ -z "$1" ]; then
    echo "Uso: $0 <azure-user@azure-vm>"
    echo "Exemplo: $0 azureuser@prysmview.eastus.cloudapp.azure.com"
    exit 1
fi

AZURE_VM=$1
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REMOTE_DIR="/opt/prysmview"

echo "🚀 Iniciando deploy do PrysmView para Azure VM: $AZURE_VM"
echo ""

# 1. Criar diretório remoto
echo "📁 Criando diretório remoto..."
ssh "$AZURE_VM" "sudo mkdir -p $REMOTE_DIR /var/lib/prysmview /etc/systemd/system"

# 2. Copiar backend Go
echo "📦 Copiando backend Go..."
scp -r "$PROJECT_DIR/server/" "$AZURE_VM:$REMOTE_DIR/"

# 3. Build remoto
echo "🔨 Buildando backend Go na VM..."
ssh "$AZURE_VM" "cd $REMOTE_DIR/server && go build -o prysmview-api ./cmd/api"

# 4. Criar serviço systemd
echo "⚙️ Criando serviço systemd..."
cat <<EOF | ssh "$AZURE_VM" "sudo tee /etc/systemd/system/prysmview-api.service"
[Unit]
Description=PrysmView Signaling API
After=network.target

[Service]
Type=simple
User=\$USER
WorkingDirectory=$REMOTE_DIR/server
ExecStart=$REMOTE_DIR/server/prysmview-api -db /var/lib/prysmview/prysmview.db -addr :8080
Restart=on-failure
RestartSec=10
Environment=PORT=8080

[Install]
WantedBy=multi-user.target
EOF

# 5. Reload systemd e iniciar serviço
echo "🔄 Iniciando serviço..."
ssh "$AZURE_VM" "sudo systemctl daemon-reload && sudo systemctl enable prysmview-api && sudo systemctl restart prysmview-api"

# 6. Verificar status
echo "✅ Verificando status do serviço..."
ssh "$AZURE_VM" "sudo systemctl status prysmview-api --no-pager"

# 7. Configurar firewall (ufw)
echo "🔒 Configurando firewall..."
ssh "$AZURE_VM" "sudo ufw allow 443/tcp && sudo ufw allow 80/tcp && sudo ufw allow 22/tcp && sudo ufw reload"

# 8. Instalar e configurar Caddy para HTTPS
echo "🔐 Instalando e configurando Caddy para HTTPS..."
ssh "$AZURE_VM" bash <<'EOF'
# Instalar Caddy
sudo apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt-get update
sudo apt-get install caddy -y

# Configurar Caddyfile
sudo tee /etc/caddy/Caddyfile > /dev/null <<'CADDYFILE'
# HTTP para HTTPS redirect
:80 {
    redir https://{host}{uri} permanent
}

# HTTPS com terminação TLS (Let's Encrypt automático)
prysmview.duckdns.org {
    header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
    header / Server ""

    # CORS para aplicacao Desktop e Web
    @cors_preflight method OPTIONS
    handle @cors_preflight {
        header Access-Control-Allow-Origin "*"
        header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS"
        header Access-Control-Allow-Headers "Content-Type, Authorization, X-Requested-With, X-Session-Code"
        respond "" 204
    }
    header Access-Control-Allow-Origin "*"
    header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS"
    header Access-Control-Allow-Headers "Content-Type, Authorization, X-Requested-With, X-Session-Code"

    reverse_proxy 127.0.0.1:8080 {
        header_up Host {upstream_hostport}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-For {remote}
        header_up X-Forwarded-Proto {scheme}
        header_up X-Forwarded-Port {server_port}
    }

    tls {
        protocols tls1.2 tls1.3
    }
}
CADDYFILE

# Reiniciar Caddy
sudo systemctl restart caddy
sudo systemctl enable caddy
EOF

echo ""
echo "✨ Deploy concluído!"
echo "📍 API rodando em: https://prysmview.duckdns.org"
echo "🏥 Health check: curl https://prysmview.duckdns.org/health"
echo ""
