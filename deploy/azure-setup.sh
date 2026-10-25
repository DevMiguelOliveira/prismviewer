#!/usr/bin/env bash
# ==============================================================================
# PrysmView - Script de Provisionamento e Deploy Automatizado (Azure Ubuntu 24.04 LTS)
# Compatível com ARM64 (Azure Standard D2alds_v7) e x86_64
# ==============================================================================
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
   echo "Este script deve ser executado como root (use: sudo bash deploy/azure-setup.sh)" >&2
   exit 1
fi

echo "=================================================================="
echo " Iniciando Deploy do PrysmView na Azure VM (Ubuntu 24.04 LTS)"
echo " Arquitetura detectada: $(uname -m)"
echo "=================================================================="

# 1. Atualizacao e pacotes base
echo "--> [1/7] Atualizando repositorios e instalando dependencias..."
export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    sqlite3 \
    ufw \
    curl \
    git \
    gzip \
    ca-certificates

# 2. Criacao de usuario de servico
echo "--> [2/7] Configurando usuario de sistema 'prysmview'..."
if ! id -u prysmview >/dev/null 2>&1; then
    useradd -r -s /usr/sbin/nologin -d /var/lib/prysmview prysmview
fi

mkdir -p /var/lib/prysmview /var/backups/prysmview /usr/local/bin /etc/prysmview
chown -R prysmview:prysmview /var/lib/prysmview
chmod 0700 /var/lib/prysmview
chmod 0700 /var/backups/prysmview

# 3. Instalacao dos toolchains (Go e Rust) se necessario
echo "--> [3/7] Verificando compiladores Go e Rust..."
ARCH=$(dpkg --print-architecture)

if ! command -v go >/dev/null 2>&1; then
    echo "Instalando Go oficial..."
    GO_VERSION="1.23.2"
    curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-${ARCH}.tar.gz" -o /tmp/go.tar.gz
    rm -rf /usr/local/go && tar -C /usr/local -xzf /tmp/go.tar.gz
    rm -f /tmp/go.tar.gz
    export PATH=$PATH:/usr/local/go/bin
    if ! grep -q "/usr/local/go/bin" /etc/profile; then
        echo 'export PATH=$PATH:/usr/local/go/bin' >> /etc/profile
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "Instalando Rust toolchain (minimal profile)..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    export PATH=$PATH:/root/.cargo/bin
fi

export PATH=$PATH:/usr/local/go/bin:/root/.cargo/bin

# 4. Compilacao nativa dos binarios
echo "--> [4/7] Compilando prysmview-api (Go) e prysmview-relay (Rust)..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "Compilando Go Signaling API..."
cd "${REPO_DIR}/server"
go build -ldflags="-s -w" -o /usr/local/bin/prysmview-api ./cmd/api/main.go
chmod 0755 /usr/local/bin/prysmview-api

echo "Compilando iroh Relay (Rust)..."
cd "${REPO_DIR}"
cargo build --release -p prysmview-relay
cp "${REPO_DIR}/target/release/prysmview-relay" /usr/local/bin/prysmview-relay
chmod 0755 /usr/local/bin/prysmview-relay

# 5. Instalacao e ativacao dos servicos systemd
echo "--> [5/7] Configurando servicos systemd..."
cp "${SCRIPT_DIR}/systemd/prysmview-api.service" /etc/systemd/system/
cp "${SCRIPT_DIR}/systemd/prysmview-relay.service" /etc/systemd/system/

systemctl daemon-reload
systemctl enable prysmview-api
systemctl enable prysmview-relay
systemctl restart prysmview-api
systemctl restart prysmview-relay

# 6. Firewall UFW
echo "--> [6/7] Ajustando regras do firewall UFW..."
bash "${SCRIPT_DIR}/setup-ufw.sh"

# 7. Backup diario do SQLite
echo "--> [7/7] Configurando rotina de backup diario..."
cp "${SCRIPT_DIR}/backup-sqlite.sh" /usr/local/bin/prysmview-backup.sh
chmod 0750 /usr/local/bin/prysmview-backup.sh

CRON_CMD="0 3 * * * /usr/local/bin/prysmview-backup.sh >> /var/log/prysmview-backup.log 2>&1"
(crontab -l 2>/dev/null | grep -Fv "/usr/local/bin/prysmview-backup.sh"; echo "$CRON_CMD") | crontab -

echo "=================================================================="
echo " Implantação concluída com sucesso na Azure VM!"
echo " Status dos serviços:"
echo "------------------------------------------------------------------"
systemctl status prysmview-api --no-pager -l
echo "------------------------------------------------------------------"
systemctl status prysmview-relay --no-pager -l
echo "=================================================================="
echo " Verificação de saúde local:"
curl -s http://127.0.0.1:8080/health || curl -s http://127.0.0.1:443/health || true
echo ""
echo " LEMBRETE AZURE NSG (Network Security Group):"
echo " Garanta que as seguintes portas estejam liberadas no portal Azure:"
echo "  - Porta 22 (TCP)   : SSH"
echo "  - Porta 80/443 (TCP): HTTP/HTTPS (Signaling API)"
echo "  - Porta 3340 (UDP) : iroh QUIC Relay"
echo "=================================================================="
