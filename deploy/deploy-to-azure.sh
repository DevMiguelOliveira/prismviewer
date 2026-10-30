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
ExecStart=$REMOTE_DIR/server/prysmview-api -db /var/lib/prysmview/prysmview.db -addr :443
Restart=on-failure
RestartSec=10
Environment=PORT=443

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
ssh "$AZURE_VM" "sudo ufw allow 443/tcp && sudo ufw allow 22/tcp && sudo ufw reload"

echo ""
echo "✨ Deploy concluído!"
echo "📍 API rodando em: https://$AZURE_VM:443"
echo "🏥 Health check: curl https://$AZURE_VM:443/health"
echo ""
